//! Sealed classifier retention; serialized intent never creates this capability.
use std::{
    fmt,
    sync::{Arc, atomic::AtomicBool},
};

use serde::{Serialize, Serializer};
use wow_core::{CanonicalResult, ContentDigest};

use crate::{
    ProjectResult,
    load::schema::classification::{
        ReviewedXmlClassificationProfile, XmlClassificationProfileSelection,
    },
    platform_source::PlatformPackageSpecialization,
};

use super::platform::invalid;

#[derive(Clone)]
pub(super) struct RetainedXmlClassification(Arc<ReviewedXmlClassificationProfile>);

impl RetainedXmlClassification {
    pub(super) fn new(
        profile: Arc<ReviewedXmlClassificationProfile>,
        packages: &PlatformPackageSpecialization,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        profile.catalog().validate_source(packages.source(), stop)?;
        let retained = Self(profile);
        retained.validate(packages)?;
        Ok(retained)
    }

    pub(super) fn owner(&self) -> &Arc<ReviewedXmlClassificationProfile> {
        &self.0
    }

    /// Immutable owner already admitted every anchor and selected member.
    /// Configuration validation compares its exact source closure without
    /// reopening bytes, parsing or adding an uncancellable catalog traversal.
    pub(super) fn validate(&self, packages: &PlatformPackageSpecialization) -> ProjectResult<()> {
        let source = packages.source().receipt();
        let catalog = self.0.catalog().receipt();
        if source.source_snapshot_id() != catalog.source_snapshot_id()
            || source.profile_digest() != catalog.source_profile_digest()
            || source.content_manifest_digest() != catalog.content_manifest_digest()
            || source.admission_digest() != catalog.admission_digest()
        {
            return Err(invalid());
        }
        Ok(())
    }
}

impl PartialEq for RetainedXmlClassification {
    fn eq(&self, other: &Self) -> bool {
        self.0.digest() == other.0.digest()
            && self.0.catalog().receipt().digest() == other.0.catalog().receipt().digest()
    }
}
impl Eq for RetainedXmlClassification {}

impl fmt::Debug for RetainedXmlClassification {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetainedXmlClassification")
            .field("profile_digest", &self.0.digest())
            .field("catalog_digest", &self.0.catalog().receipt().digest())
            .finish_non_exhaustive()
    }
}

impl Serialize for RetainedXmlClassification {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Binding<'a> {
            selection: &'a XmlClassificationProfileSelection,
            digest: ContentDigest<CanonicalResult>,
        }
        Binding {
            selection: self.0.selection(),
            digest: self.0.digest(),
        }
        .serialize(serializer)
    }
}
