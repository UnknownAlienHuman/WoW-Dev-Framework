use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CommitReceipt, GarbageCollectionReceipt,
    IntegrityReport, LeaseId, LeaseRecord, LogicalEpoch, LogicalManifest, ObjectId, ObjectRecord,
    PendingObject, Store, WriteBatch,
};

use crate::{GraphError, GraphErrorCode, GraphPublicationKey, GraphResult, GraphSnapshot};

pub const GRAPH_STORE_SCHEMA: &str = "wow-graph/store/e2-a/1";
pub const GRAPH_SNAPSHOT_OBJECT_KIND: &str = "wow.graph.snapshot";
pub const GRAPH_SNAPSHOT_OBJECT_SCHEMA_VERSION: u32 = 1;
const CURRENT_CATALOG: &str = "graph.current";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredGraphSnapshot {
    object_id: ObjectId,
    snapshot_id: crate::GraphSnapshotId,
    commit_receipt: CommitReceipt,
}

impl StoredGraphSnapshot {
    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &crate::GraphSnapshotId {
        &self.snapshot_id
    }

    #[must_use]
    pub fn commit_receipt(&self) -> &CommitReceipt {
        &self.commit_receipt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedGraphSnapshot {
    publication_key: GraphPublicationKey,
    object_id: ObjectId,
    snapshot: GraphSnapshot,
}

impl PublishedGraphSnapshot {
    #[must_use]
    pub fn publication_key(&self) -> &GraphPublicationKey {
        &self.publication_key
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn snapshot(&self) -> &GraphSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn into_snapshot(self) -> GraphSnapshot {
        self.snapshot
    }
}

pub struct PersistentGraphStore<'store> {
    store: &'store mut Store,
}

impl<'store> PersistentGraphStore<'store> {
    #[must_use]
    pub fn new(store: &'store mut Store) -> Self {
        Self { store }
    }

    pub fn store_snapshot(&mut self, snapshot: &GraphSnapshot) -> GraphResult<StoredGraphSnapshot> {
        snapshot.validate()?;
        let pending = self.pending(snapshot)?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        let commit_receipt = self.store.commit(batch)?;
        Ok(StoredGraphSnapshot {
            object_id,
            snapshot_id: snapshot.snapshot_id().clone(),
            commit_receipt,
        })
    }

    pub fn publish_current(
        &mut self,
        publication_key: GraphPublicationKey,
        snapshot: &GraphSnapshot,
        expectation: CatalogExpectation,
    ) -> GraphResult<StoredGraphSnapshot> {
        snapshot.validate()?;
        if snapshot.universe() != publication_key.universe() {
            return Err(GraphError::new(
                GraphErrorCode::UniverseMismatch,
                "graph publication key belongs to another universe",
            ));
        }
        let pending = self.pending(snapshot)?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        batch.add_catalog_mutation(CatalogMutation::set(
            current_catalog()?,
            publication_key.catalog_path().clone(),
            expectation,
            object_id.clone(),
        ))?;
        let commit_receipt = self.store.commit(batch)?;
        Ok(StoredGraphSnapshot {
            object_id,
            snapshot_id: snapshot.snapshot_id().clone(),
            commit_receipt,
        })
    }

    pub fn read_exact(&self, object_id: &ObjectId) -> GraphResult<Option<GraphSnapshot>> {
        let Some(record) = self.store.object(object_id)? else {
            return Ok(None);
        };
        Ok(Some(decode_snapshot(&record)?))
    }

    pub fn read_current(
        &self,
        publication_key: &GraphPublicationKey,
    ) -> GraphResult<Option<PublishedGraphSnapshot>> {
        let Some(entry) = self
            .store
            .catalog_entry(&current_catalog()?, publication_key.catalog_path())?
        else {
            return Ok(None);
        };
        let object_id = entry.object_id().clone();
        let snapshot = self.read_exact(&object_id)?.ok_or_else(|| {
            GraphError::new(
                GraphErrorCode::SnapshotInvalid,
                "published graph snapshot object is missing",
            )
        })?;
        if snapshot.universe() != publication_key.universe() {
            return Err(GraphError::new(
                GraphErrorCode::UniverseMismatch,
                "published graph snapshot belongs to another universe",
            ));
        }
        Ok(Some(PublishedGraphSnapshot {
            publication_key: publication_key.clone(),
            object_id,
            snapshot,
        }))
    }

    pub fn unpublish_current(
        &mut self,
        publication_key: &GraphPublicationKey,
        expected: ObjectId,
    ) -> GraphResult<CommitReceipt> {
        let mut batch = WriteBatch::new();
        batch.add_catalog_mutation(CatalogMutation::delete(
            current_catalog()?,
            publication_key.catalog_path().clone(),
            expected,
        ))?;
        Ok(self.store.commit(batch)?)
    }

    pub fn retain(
        &mut self,
        lease_id: LeaseId,
        object_id: ObjectId,
        holder: impl Into<Box<str>>,
        expires_after: LogicalEpoch,
    ) -> GraphResult<LeaseRecord> {
        Ok(self
            .store
            .acquire_lease(lease_id, object_id, holder, expires_after)?)
    }

    pub fn renew_retention(
        &mut self,
        lease_id: &LeaseId,
        holder: &str,
        expected_expiry: LogicalEpoch,
        new_expiry: LogicalEpoch,
    ) -> GraphResult<LeaseRecord> {
        Ok(self
            .store
            .renew_lease(lease_id, holder, expected_expiry, new_expiry)?)
    }

    pub fn release_retention(&mut self, lease_id: &LeaseId, holder: &str) -> GraphResult<()> {
        Ok(self.store.release_lease(lease_id, holder)?)
    }

    pub fn collect_garbage(
        &mut self,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> GraphResult<GarbageCollectionReceipt> {
        Ok(self.store.collect_garbage(now, max_deletes)?)
    }

    pub fn validate_integrity(&self, max_objects: u32) -> GraphResult<IntegrityReport> {
        Ok(self.store.validate_integrity(max_objects)?)
    }

    pub fn logical_manifest(&self) -> GraphResult<LogicalManifest> {
        Ok(self.store.logical_manifest()?)
    }

    fn pending(&self, snapshot: &GraphSnapshot) -> GraphResult<PendingObject> {
        Ok(PendingObject::from_json(
            GRAPH_SNAPSHOT_OBJECT_KIND,
            GRAPH_SNAPSHOT_OBJECT_SCHEMA_VERSION,
            snapshot,
            self.store.configuration().limits(),
        )?)
    }
}

fn current_catalog() -> GraphResult<CatalogName> {
    Ok(CatalogName::new(CURRENT_CATALOG)?)
}

fn decode_snapshot(record: &ObjectRecord) -> GraphResult<GraphSnapshot> {
    if record.kind() != GRAPH_SNAPSHOT_OBJECT_KIND {
        return Err(GraphError::new(
            GraphErrorCode::ArtifactKindMismatch,
            "stored object is not a graph snapshot",
        ));
    }
    if record.schema_version() != GRAPH_SNAPSHOT_OBJECT_SCHEMA_VERSION {
        return Err(GraphError::new(
            GraphErrorCode::ArtifactSchemaMismatch,
            "stored graph snapshot schema is unsupported",
        ));
    }
    let snapshot: GraphSnapshot = record.decode().map_err(|_| {
        GraphError::new(
            GraphErrorCode::ArtifactDecodeFailed,
            "stored graph snapshot cannot be decoded",
        )
    })?;
    snapshot.validate()?;
    Ok(snapshot)
}
