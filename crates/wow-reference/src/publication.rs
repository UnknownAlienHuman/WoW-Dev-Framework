//! Side-effect-free typed preparation for service-owned durable orchestration.

use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, ObjectId, PendingObject, StoreLimits,
    WriteBatch,
};

use crate::{
    ReferenceView,
    persistent::{
        REFERENCE_VIEW_OBJECT_KIND, REFERENCE_VIEW_OBJECT_SCHEMA_VERSION,
        ReferencePublicationKey, ReferenceStoreError, ReferenceStoreResult,
    },
};

const CURRENT_CATALOG: &str = "reference.current";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedReferencePublication {
    publication_key: ReferencePublicationKey,
    expectation: CatalogExpectation,
    object: PendingObject,
}

impl PreparedReferencePublication {
    pub fn new(
        publication_key: ReferencePublicationKey,
        view: &ReferenceView,
        expectation: CatalogExpectation,
        limits: StoreLimits,
    ) -> ReferenceStoreResult<Self> {
        let object = PendingObject::from_json(
            REFERENCE_VIEW_OBJECT_KIND,
            REFERENCE_VIEW_OBJECT_SCHEMA_VERSION,
            view,
            limits,
        )?;
        Ok(Self {
            publication_key,
            expectation,
            object,
        })
    }

    #[must_use]
    pub fn publication_key(&self) -> &ReferencePublicationKey {
        &self.publication_key
    }

    #[must_use]
    pub fn expectation(&self) -> &CatalogExpectation {
        &self.expectation
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        self.object.object_id()
    }

    #[must_use]
    pub fn expectation_matches(&self, current: Option<&ObjectId>) -> bool {
        match &self.expectation {
            CatalogExpectation::Absent => current.is_none(),
            CatalogExpectation::Exact(expected) => current == Some(expected),
        }
    }

    pub fn into_batch(self) -> ReferenceStoreResult<WriteBatch> {
        let mut batch = WriteBatch::new();
        let object_id = self.object.object_id().clone();
        batch.add_object(self.object)?;
        batch.add_catalog_mutation(CatalogMutation::set(
            CatalogName::new(CURRENT_CATALOG)?,
            self.publication_key.catalog_path().clone(),
            self.expectation,
            object_id,
        ))?;
        Ok(batch)
    }
}

impl From<wow_store::StoreError> for ReferenceStoreError {
    fn from(source: wow_store::StoreError) -> Self {
        crate::persistent::map_store_error(source)
    }
}
