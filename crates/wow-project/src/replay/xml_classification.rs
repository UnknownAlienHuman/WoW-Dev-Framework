//! Intent-only replay; re-admit the catalog and profile on the rebuilt source Arc.
use std::sync::{Arc, atomic::AtomicBool};

use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest};

use crate::{
    ProjectResult,
    load::schema::{
        XmlSchemaSelection, admit_xml_schema,
        classification::{
            ReviewedXmlClassificationProfile, XmlClassificationProfileSelection,
            admit_xml_classification_profile,
        },
    },
    platform_source::AdmittedPlatformSource,
};

use super::invalid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplayXmlClassification {
    schema_selection: XmlSchemaSelection,
    profile_selection: XmlClassificationProfileSelection,
    expected_catalog_digest: ContentDigest<CanonicalResult>,
    expected_profile_digest: ContentDigest<CanonicalResult>,
}

impl ReplayXmlClassification {
    pub(super) fn capture(profile: &ReviewedXmlClassificationProfile) -> Self {
        let catalog = profile.catalog().receipt();
        Self {
            schema_selection: XmlSchemaSelection {
                source_snapshot_id: catalog.source_snapshot_id().to_owned(),
                profile_digest: catalog.source_profile_digest(),
                content_manifest_digest: catalog.content_manifest_digest(),
                admission_digest: catalog.admission_digest(),
                members: catalog.members().to_vec(),
            },
            profile_selection: profile.selection().clone(),
            expected_catalog_digest: catalog.digest(),
            expected_profile_digest: profile.digest(),
        }
    }

    pub(super) fn rebuild(
        &self,
        source: &Arc<AdmittedPlatformSource>,
        stop: &AtomicBool,
    ) -> ProjectResult<Arc<ReviewedXmlClassificationProfile>> {
        crate::analyzer::checkpoint(stop)?;
        let catalog = Arc::new(admit_xml_schema(source, &self.schema_selection, stop)?);
        if catalog.receipt().digest() != self.expected_catalog_digest {
            return Err(invalid());
        }
        let profile =
            admit_xml_classification_profile(&catalog, source, &self.profile_selection, stop)?;
        if profile.digest() != self.expected_profile_digest {
            return Err(invalid());
        }
        crate::analyzer::checkpoint(stop)?;
        Ok(profile)
    }
}
