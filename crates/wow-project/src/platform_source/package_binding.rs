//! Sealed identity binding between admitted source and native package owners.

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest};

use super::{AdmittedPlatformSource, PlatformTarget, invalid};
use crate::{
    ProjectPhase, ProjectResult,
    identity::canonical_digest,
    load::{ProjectPackageLoadPlan, ProjectPackageMainPlan},
};

const SCHEMA: &str = "wow-project/platform-package-binding/1";

/// Serialize-only evidence of an actual source/load/Main owner binding.
/// This identity grants no project, analyzer, publication or runtime authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformPackageBinding {
    schema: &'static str,
    profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    source_snapshot_id: Box<str>,
    universe_id: Box<str>,
    target: PlatformTarget,
    load_digest: ContentDigest<CanonicalResult>,
    main_digest: ContentDigest<CanonicalResult>,
    binding_digest: ContentDigest<CanonicalResult>,
}

impl PlatformPackageBinding {
    pub(super) fn new(
        source: &AdmittedPlatformSource,
        load: &ProjectPackageLoadPlan,
        main: &ProjectPackageMainPlan,
    ) -> ProjectResult<Self> {
        let profile = source.profile();
        profile.validate()?;
        let receipt = source.receipt();
        if receipt.profile_digest() != profile.digest()
            || &receipt.inventory().target != profile.target()
        {
            return Err(invalid(
                "platform source receipt and profile binding differ",
            ));
        }
        load.validate_profile(&profile.target().reference_profile)?;
        main.validate_load_plan(load)?;

        let profile_digest = receipt.profile_digest();
        let content_manifest_digest = receipt.content_manifest_digest();
        let admission_digest = receipt.admission_digest();
        let source_snapshot_id: Box<str> = receipt.source_snapshot_id().into();
        let universe_id: Box<str> =
            format!("blizzard_ui_source:{profile_digest}:{source_snapshot_id}").into();
        let target = profile.target().clone();
        let load_digest = load.digest();
        let main_digest = main.digest();
        let binding_digest = canonical_digest(
            SCHEMA,
            &BindingIdentity {
                schema: SCHEMA,
                profile_digest,
                content_manifest_digest,
                admission_digest,
                source_snapshot_id: &source_snapshot_id,
                universe_id: &universe_id,
                target: &target,
                load_digest,
                main_digest,
            },
            ProjectPhase::Inventory,
        )?;
        Ok(Self {
            schema: SCHEMA,
            profile_digest,
            content_manifest_digest,
            admission_digest,
            source_snapshot_id,
            universe_id,
            target,
            load_digest,
            main_digest,
            binding_digest,
        })
    }

    /// Reconstruct the binding from the exact real owners and compare all fields.
    pub fn validate(
        &self,
        source: &AdmittedPlatformSource,
        load: &ProjectPackageLoadPlan,
        main: &ProjectPackageMainPlan,
    ) -> ProjectResult<()> {
        if self != &Self::new(source, load, main)? {
            return Err(invalid(
                "platform package binding does not match its owners",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn profile_digest(&self) -> ContentDigest<CanonicalResult> {
        self.profile_digest
    }

    #[must_use]
    pub const fn content_manifest_digest(&self) -> ContentDigest<CanonicalResult> {
        self.content_manifest_digest
    }

    #[must_use]
    pub const fn admission_digest(&self) -> ContentDigest<CanonicalResult> {
        self.admission_digest
    }

    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        &self.source_snapshot_id
    }

    #[must_use]
    pub fn universe_id(&self) -> &str {
        &self.universe_id
    }

    #[must_use]
    pub const fn target(&self) -> &PlatformTarget {
        &self.target
    }

    #[must_use]
    pub const fn load_digest(&self) -> ContentDigest<CanonicalResult> {
        self.load_digest
    }

    #[must_use]
    pub const fn main_digest(&self) -> ContentDigest<CanonicalResult> {
        self.main_digest
    }

    #[must_use]
    pub const fn binding_digest(&self) -> ContentDigest<CanonicalResult> {
        self.binding_digest
    }
}

#[derive(Serialize)]
struct BindingIdentity<'a> {
    schema: &'static str,
    profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    source_snapshot_id: &'a str,
    universe_id: &'a str,
    target: &'a PlatformTarget,
    load_digest: ContentDigest<CanonicalResult>,
    main_digest: ContentDigest<CanonicalResult>,
}
