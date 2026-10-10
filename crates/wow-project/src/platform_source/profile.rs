//! Validated, finite configuration for explicit local raw-source admission.
use super::{budget, invalid, path, serialized_size, under};
use crate::identity::canonical_digest;
use crate::{ProjectPhase, ProjectResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use wow_core::{
    CanonicalResult, ContentDigest, ProfileId, ProfileIdentity, ProfileKind, ReferenceGenerationId,
};

const SCHEMA: &str = "wow-project/platform-source-profile/1";
const MAX_PROFILE_BYTES: usize = 256 * 1024;
const MAX_ROOTS: usize = 64;
const MAX_TOCS: usize = 1024;
const MAX_EXCLUSIONS: usize = 1024;
const MAX_ENTRIES: usize = 200_000;
const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024 * 1024;

/// Evidence class; fixture bytes cannot become mirror evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformSourceClass {
    VendorUiSourceMirror,
    SyntheticFixture,
}

/// Explicit target identity, without build or flavor inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformTarget {
    pub product: String,
    pub channel: String,
    pub reference_profile: ProfileIdentity,
    pub reference_generation: ReferenceGenerationId,
}
impl PlatformTarget {
    pub(super) fn validate(&self) -> ProjectResult<()> {
        if !target_id(&self.product) || !target_id(&self.channel) {
            return Err(invalid("invalid platform product or channel identity"));
        }
        if self.reference_profile.schema_versions().len() > 64 {
            return Err(budget("platform target has too many schema versions"));
        }
        serialized_size(self, MAX_PROFILE_BYTES)?;
        self.reference_profile
            .validate()
            .map_err(|_| invalid("invalid platform reference profile"))
    }
}

/// A snapshot-relative root and its explicitly selected TOCs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformRootSpec {
    pub root: String,
    pub selected_tocs: Vec<String>,
}

/// An exact-file exclusion, never a subtree, glob or executable predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileExclusion {
    pub path: String,
}

/// Finite raw-byte limits; these do not widen downstream Lua/parser limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAdmissionLimits {
    pub max_entries: usize,
    pub max_total_bytes: u64,
    pub max_file_bytes: u64,
    pub max_manifest_bytes: usize,
}
impl SourceAdmissionLimits {
    pub fn new(
        max_entries: usize,
        max_total_bytes: u64,
        max_file_bytes: u64,
        max_manifest_bytes: usize,
    ) -> ProjectResult<Self> {
        let limits = Self {
            max_entries,
            max_total_bytes,
            max_file_bytes,
            max_manifest_bytes,
        };
        limits.validate()?;
        Ok(limits)
    }
    pub(super) fn validate(&self) -> ProjectResult<()> {
        if self.max_entries == 0
            || self.max_total_bytes == 0
            || self.max_file_bytes == 0
            || self.max_manifest_bytes == 0
            || self.max_file_bytes > self.max_total_bytes
        {
            return Err(invalid("zero or inconsistent platform source limits"));
        }
        if self.max_entries > MAX_ENTRIES
            || self.max_total_bytes > MAX_TOTAL_BYTES
            || self.max_file_bytes > MAX_FILE_BYTES
            || self.max_manifest_bytes > MAX_MANIFEST_BYTES
        {
            return Err(budget("platform source limits exceed raw owner caps"));
        }
        Ok(())
    }
}

/// Data-only input; admission validates direct Rust values and decoded values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlizzardUiSourceProfileRequest {
    pub profile_id: ProfileId,
    pub source_class: PlatformSourceClass,
    pub target: PlatformTarget,
    pub roots: Vec<PlatformRootSpec>,
    pub exclusions: Vec<ProfileExclusion>,
    pub limits: SourceAdmissionLimits,
}

/// Immutable profile receipt. No source, Git membership or client attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlizzardUiSourceProfile {
    schema: &'static str,
    request: BlizzardUiSourceProfileRequest,
    digest: ContentDigest<CanonicalResult>,
}
impl BlizzardUiSourceProfile {
    pub fn new(mut request: BlizzardUiSourceProfileRequest) -> ProjectResult<Self> {
        serialized_size(&request, MAX_PROFILE_BYTES)?;
        canonicalize(&mut request)?;
        let projection = ProfileProjection {
            schema: SCHEMA,
            request: &request,
        };
        serialized_size(&projection, MAX_PROFILE_BYTES)?;
        let digest = canonical_digest(SCHEMA, &projection, ProjectPhase::Inventory)?;
        let profile = Self {
            schema: SCHEMA,
            request,
            digest,
        };
        serialized_size(&profile, MAX_PROFILE_BYTES)?;
        Ok(profile)
    }
    pub fn from_json(bytes: &[u8]) -> ProjectResult<Self> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err(budget("platform profile envelope exceeds its limit"));
        }
        let request = serde_json::from_slice(bytes)
            .map_err(|_| invalid("invalid platform profile request"))?;
        Self::new(request)
    }
    pub fn validate(&self) -> ProjectResult<()> {
        serialized_size(self, MAX_PROFILE_BYTES)?;
        let expected = Self::new(self.request.clone())?;
        if self != &expected {
            return Err(invalid("platform profile identity is inconsistent"));
        }
        Ok(())
    }
    pub fn profile_id(&self) -> &ProfileId {
        &self.request.profile_id
    }
    pub fn source_class(&self) -> PlatformSourceClass {
        self.request.source_class
    }
    pub fn target(&self) -> &PlatformTarget {
        &self.request.target
    }
    pub fn roots(&self) -> &[PlatformRootSpec] {
        &self.request.roots
    }
    pub fn exclusions(&self) -> &[ProfileExclusion] {
        &self.request.exclusions
    }
    pub fn limits(&self) -> SourceAdmissionLimits {
        self.request.limits
    }
    pub fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
}

#[derive(Serialize)]
struct ProfileProjection<'a> {
    schema: &'static str,
    request: &'a BlizzardUiSourceProfileRequest,
}

fn canonicalize(request: &mut BlizzardUiSourceProfileRequest) -> ProjectResult<()> {
    request.target.validate()?;
    request.limits.validate()?;
    if request.profile_id.namespace() != request.target.reference_profile.profile_id().namespace() {
        return Err(invalid(
            "platform source profile label has a conflicting namespace",
        ));
    }
    let kind = request.target.reference_profile.profile_kind();
    if !matches!(
        (request.source_class, kind),
        (PlatformSourceClass::SyntheticFixture, ProfileKind::Fixture)
            | (
                PlatformSourceClass::VendorUiSourceMirror,
                ProfileKind::Release
            )
    ) {
        return Err(invalid(
            "platform source class and reference profile differ",
        ));
    }
    if request.roots.is_empty() {
        return Err(invalid("platform source profile requires explicit roots"));
    }
    if request.roots.len() > MAX_ROOTS || request.exclusions.len() > MAX_EXCLUSIONS {
        return Err(budget("platform profile has too many roots or exclusions"));
    }

    let mut roots = BTreeSet::new();
    let mut tocs = BTreeSet::new();
    let mut toc_count = 0_usize;
    for root in &mut request.roots {
        path(&root.root)?;
        let folded = root.root.to_ascii_lowercase();
        if roots.iter().any(|other: &String| {
            other == &folded || under(other, &folded) || under(&folded, other)
        }) {
            return Err(invalid("platform roots collide or overlap"));
        }
        roots.insert(folded);
        toc_count = toc_count
            .checked_add(root.selected_tocs.len())
            .filter(|count| *count <= MAX_TOCS)
            .ok_or_else(|| budget("platform profile has too many selected TOCs"))?;
        for toc in &root.selected_tocs {
            path(toc)?;
            let folded = toc.to_ascii_lowercase();
            if toc == &root.root
                || !under(toc, &root.root)
                || !folded.ends_with(".toc")
                || !tocs.insert(folded)
            {
                return Err(invalid("invalid or colliding platform TOC selection"));
            }
        }
        root.selected_tocs.sort();
    }

    let mut exclusions = BTreeSet::new();
    for exclusion in &request.exclusions {
        path(&exclusion.path)?;
        let folded = exclusion.path.to_ascii_lowercase();
        if tocs.contains(&folded)
            || !exclusions.insert(folded)
            || !request
                .roots
                .iter()
                .any(|root| exclusion.path != root.root && under(&exclusion.path, &root.root))
        {
            return Err(invalid("invalid or colliding platform file exclusion"));
        }
    }
    request.roots.sort_by(|a, b| a.root.cmp(&b.root));
    request.exclusions.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(())
}

fn target_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
