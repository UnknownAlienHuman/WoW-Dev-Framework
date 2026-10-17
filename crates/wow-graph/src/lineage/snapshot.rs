//! Separately stored immutable Candidate overlay. Source retention and durable
//! operation orchestration remain the Service/Store consumers' responsibility.
use super::*;
use serde::{Deserialize, Serialize};
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, ObjectId, OperationId,
    OperationState, PendingObject, RequestDigest, Store, WriteBatch,
};

pub const LINEAGE_STORE_PROFILE: &str = "wow-graph.lineage-project-span-store.v1";
pub const LINEAGE_SNAPSHOT_KIND: &str = "wow.graph.lineage.candidate.snapshot";
pub const LINEAGE_SNAPSHOT_SCHEMA: &str = "wow-graph/lineage-candidate-snapshot/1";
pub const SOURCE_LINEAGE_STORE_PROFILE: &str = "wow-graph.lineage-source-span-store.v1";
pub const SOURCE_LINEAGE_SNAPSHOT_SCHEMA: &str = "wow-graph/lineage-source-candidate-snapshot/1";
const REGISTRY: &str = "wow-graph/lineage-project-span-candidate-registry/1";
const SOURCE_REGISTRY: &str = "wow-graph/lineage-source-span-candidate-registry/1";
#[derive(Clone, Copy)]
struct StorageProfile {
    profile: &'static str,
    schema: &'static str,
    registry: &'static str,
    store: &'static str,
    validation_scope: &'static str,
}
const PROJECT_STORAGE: StorageProfile = StorageProfile {
    profile: PROJECT_SPAN_LINEAGE_PROFILE,
    schema: LINEAGE_SNAPSHOT_SCHEMA,
    registry: REGISTRY,
    store: LINEAGE_STORE_PROFILE,
    validation_scope: "native-project-source-span-candidates-only",
};
const SOURCE_STORAGE: StorageProfile = StorageProfile {
    profile: SOURCE_SPAN_LINEAGE_PROFILE,
    schema: SOURCE_LINEAGE_SNAPSHOT_SCHEMA,
    registry: SOURCE_REGISTRY,
    store: SOURCE_LINEAGE_STORE_PROFILE,
    validation_scope: "native-blizzard-ui-source-span-candidates-only",
};
fn storage_profile(profile: &str) -> GraphResult<StorageProfile> {
    match profile {
        PROJECT_SPAN_LINEAGE_PROFILE => Ok(PROJECT_STORAGE),
        SOURCE_SPAN_LINEAGE_PROFILE => Ok(SOURCE_STORAGE),
        _ => Err(invalid()),
    }
}
const INACTIVE: &str = "lineage.inactive";
const SEALED: &str = "lineage.snapshots";
const VALIDATION: &str = "lineage.validation";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SnapshotRecord {
    pub(super) schema: Box<str>,
    pub(super) profile_id: Box<str>,
    pub(super) registry_id: Box<str>,
    pub(super) snapshot_id: Box<str>,
    pub(super) input_partition_id: Box<str>,
    pub(super) limits: LineageLimits,
    pub(super) input: ProjectFingerprintPartition,
    pub(super) candidates: LineageCandidateSet,
    pub(super) accepted_assertion_count: usize,
    pub(super) promotion: Box<str>,
    pub(super) negative_authority: bool,
}
#[derive(Serialize)]
struct Identity<'a> {
    schema: &'a str,
    profile_id: &'a str,
    registry_id: &'a str,
    input_partition_id: &'a str,
    limits: LineageLimits,
    input: &'a ProjectFingerprintPartition,
    candidates: &'a LineageCandidateSet,
    accepted_assertion_count: usize,
    promotion: &'a str,
    negative_authority: bool,
}
impl SnapshotRecord {
    fn identity(&self) -> Identity<'_> {
        Identity {
            schema: &self.schema,
            profile_id: &self.profile_id,
            registry_id: &self.registry_id,
            input_partition_id: &self.input_partition_id,
            limits: self.limits,
            input: &self.input,
            candidates: &self.candidates,
            accepted_assertion_count: self.accepted_assertion_count,
            promotion: &self.promotion,
            negative_authority: self.negative_authority,
        }
    }
    fn validate_identity(&self, stop: &AtomicBool) -> GraphResult<()> {
        checkpoint(stop)?;
        let profile = storage_profile(&self.profile_id)?;
        let (input_profile, input_schema, input_producer) =
            super::source_model::selected_input_profile(&self.input)?;
        if self.schema.as_ref() != profile.schema
            || self.registry_id.as_ref() != profile.registry
            || input_profile != profile.profile
            || self.input.schema.as_ref() != input_schema
            || self.input.producer_id.as_ref() != input_producer
            || self.candidates.profile_id.as_ref() != profile.profile
            || self.candidates.input_partition_id != self.input_partition_id
            || self.accepted_assertion_count != 0
            || self.promotion.as_ref() != "not_evaluated"
            || self.negative_authority
            || self.snapshot_id != identity("wow-graph/lineage-snapshot/1", &self.identity())?
        {
            return Err(GraphError::new(
                GraphErrorCode::LineageSnapshotInvalid,
                "lineage snapshot identity or proof ceiling differs",
            ));
        }
        Ok(())
    }
}

/// Only native membership-qualified inputs can construct this prepared overlay.
/// It is not a sealed publication or a retained source-owner view.
pub struct PreparedLineageSnapshot {
    pub(super) record: SnapshotRecord,
    usage: LineageUsage,
}
impl PreparedLineageSnapshot {
    pub fn build(input: ValidatedLineageInput, stop: &AtomicBool) -> GraphResult<Self> {
        let candidates = generate_lineage_candidate_components(&input, stop)?;
        let profile = storage_profile(&input.data.binding.profile_id)?;
        let mut ledger = OperationBudget::resume(input.limits, candidates.usage(), stop)?;
        let mut record = SnapshotRecord {
            schema: profile.schema.into(),
            profile_id: profile.profile.into(),
            registry_id: profile.registry.into(),
            snapshot_id: "".into(),
            input_partition_id: input.partition_id,
            limits: input.limits,
            input: input.data,
            candidates,
            accepted_assertion_count: 0,
            promotion: "not_evaluated".into(),
            negative_authority: false,
        };
        ledger.metadata(&record.identity())?;
        record.snapshot_id = identity("wow-graph/lineage-snapshot/1", &record.identity())?;
        ledger.metadata(&record.snapshot_id)?;
        record.validate_identity(stop)?;
        Ok(Self {
            record,
            usage: ledger.usage,
        })
    }
    pub fn snapshot_id(&self) -> &str {
        &self.record.snapshot_id
    }
    pub fn input(&self) -> &ProjectFingerprintPartition {
        &self.record.input
    }
    pub fn candidates(&self) -> &LineageCandidateSet {
        &self.record.candidates
    }
    pub fn usage(&self) -> LineageUsage {
        self.usage
    }
}

pub struct StoredInactiveLineageSnapshot {
    publication_id: Box<str>,
    object_id: ObjectId,
    snapshot_id: Box<str>,
}
impl StoredInactiveLineageSnapshot {
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }
}
/// Minted only by exact inactive-catalog/object readback and native expected data.
pub struct ValidatedStoredLineageSnapshot {
    stored: StoredInactiveLineageSnapshot,
}
pub struct LineageSealPlan {
    pub batch: WriteBatch,
    pub handle: SealedLineageHandle,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealedLineageHandle {
    pub schema: Box<str>,
    pub publication_id: Box<str>,
    pub snapshot_id: Box<str>,
    pub object_id: ObjectId,
    pub validation_object_id: ObjectId,
}

pub struct PersistentLineageStore<'a> {
    pub(super) store: &'a mut Store,
    profile: StorageProfile,
}
impl<'a> PersistentLineageStore<'a> {
    pub fn new(store: &'a mut Store) -> GraphResult<Self> {
        Self::open_profile(store, PROJECT_STORAGE)
    }
    pub fn new_source(store: &'a mut Store) -> GraphResult<Self> {
        Self::open_profile(store, SOURCE_STORAGE)
    }
    fn open_profile(store: &'a mut Store, profile: StorageProfile) -> GraphResult<Self> {
        if store.configuration().profile_id() != profile.store {
            return Err(GraphError::new(
                GraphErrorCode::LineageProfileInvalid,
                "lineage store profile differs",
            ));
        }
        store.store_instance_id()?;
        Ok(Self { store, profile })
    }
    pub fn store_inactive(
        &mut self,
        publication_id: &str,
        prepared: &PreparedLineageSnapshot,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
        stop: &AtomicBool,
    ) -> GraphResult<StoredInactiveLineageSnapshot> {
        checkpoint(stop)?;
        prepared.record.validate_identity(stop)?;
        if prepared.record.profile_id.as_ref() != self.profile.profile {
            return Err(invalid());
        }
        let operation = self.store.operation(operation_id)?.ok_or_else(invalid)?;
        if operation.state() != OperationState::Prepared
            || operation.request_digest() != request_digest
        {
            return Err(invalid());
        }
        let pending = PendingObject::from_json(
            LINEAGE_SNAPSHOT_KIND,
            1,
            &prepared.record,
            self.store.configuration().limits(),
        )?;
        let object_id = pending.object_id().clone();
        let path = path(publication_id)?;
        match self.store.catalog_entry(&catalog(INACTIVE)?, &path)? {
            Some(entry) if entry.object_id() == &object_id => {}
            Some(_) => {
                return Err(GraphError::new(
                    GraphErrorCode::LineageSnapshotInvalid,
                    "inactive publication already has another object",
                ));
            }
            None => {
                let mut batch = WriteBatch::new();
                batch.add_object(pending)?;
                batch.add_catalog_mutation(CatalogMutation::set(
                    catalog(INACTIVE)?,
                    path,
                    CatalogExpectation::Absent,
                    object_id.clone(),
                ))?;
                self.store.commit(batch)?;
            }
        }
        checkpoint(stop)?;
        Ok(StoredInactiveLineageSnapshot {
            publication_id: publication_id.into(),
            object_id,
            snapshot_id: prepared.snapshot_id().into(),
        })
    }
    /// Call after checked close/reopen of the Store for the physical readback.
    /// Equality is against native qualified construction, never decoded flags.
    pub fn validate_inactive(
        &self,
        stored: StoredInactiveLineageSnapshot,
        expected: &PreparedLineageSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<ValidatedStoredLineageSnapshot> {
        checkpoint(stop)?;
        let entry = self
            .store
            .catalog_entry(&catalog(INACTIVE)?, &path(&stored.publication_id)?)?
            .ok_or_else(invalid)?;
        if entry.object_id() != &stored.object_id
            || stored.snapshot_id.as_ref() != expected.snapshot_id()
        {
            return Err(invalid());
        }
        let record = self.read_record(&stored.object_id, stop)?;
        if record != expected.record {
            return Err(GraphError::new(
                GraphErrorCode::LineageSnapshotInvalid,
                "stored lineage differs from native qualified inputs",
            ));
        }
        Ok(ValidatedStoredLineageSnapshot { stored })
    }
    pub fn plan_seal(
        &self,
        validated: ValidatedStoredLineageSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<LineageSealPlan> {
        checkpoint(stop)?;
        let stored = validated.stored;
        #[derive(Serialize)]
        struct Report<'a> {
            schema: &'static str,
            snapshot_id: &'a str,
            object_id: &'a ObjectId,
            status: &'static str,
            scope: &'static str,
        }
        let report = PendingObject::from_json(
            "wow.graph.lineage.validation",
            1,
            &Report {
                schema: "wow-graph/lineage-candidate-validation/1",
                snapshot_id: &stored.snapshot_id,
                object_id: &stored.object_id,
                status: "valid",
                scope: self.profile.validation_scope,
            },
            self.store.configuration().limits(),
        )?;
        let validation_object_id = report.object_id().clone();
        let p = path(&stored.publication_id)?;
        let mut batch = WriteBatch::new();
        batch.add_object(report)?;
        batch.add_catalog_mutation(CatalogMutation::delete(
            catalog(INACTIVE)?,
            p.clone(),
            stored.object_id.clone(),
        ))?;
        batch.add_catalog_mutation(CatalogMutation::set(
            catalog(SEALED)?,
            p.clone(),
            CatalogExpectation::Absent,
            stored.object_id.clone(),
        ))?;
        batch.add_catalog_mutation(CatalogMutation::set(
            catalog(VALIDATION)?,
            p,
            CatalogExpectation::Absent,
            validation_object_id.clone(),
        ))?;
        Ok(LineageSealPlan {
            batch,
            handle: SealedLineageHandle {
                schema: "wow-graph/lineage-candidate-handle/1".into(),
                publication_id: stored.publication_id,
                snapshot_id: stored.snapshot_id,
                object_id: stored.object_id,
                validation_object_id,
            },
        })
    }
    pub fn validate_sealed(
        &self,
        handle: &SealedLineageHandle,
        expected: &PreparedLineageSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<()> {
        self.validated_record(handle, expected, stop).map(|_| ())
    }
    pub(super) fn validated_record(
        &self,
        handle: &SealedLineageHandle,
        expected: &PreparedLineageSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<SnapshotRecord> {
        checkpoint(stop)?;
        if handle.schema.as_ref() != "wow-graph/lineage-candidate-handle/1"
            || handle.snapshot_id.as_ref() != expected.snapshot_id()
        {
            return Err(invalid());
        }
        self.require_roots(handle)?;
        let record = self.read_record(&handle.object_id, stop)?;
        if record != expected.record {
            return Err(GraphError::new(
                GraphErrorCode::LineageSnapshotInvalid,
                "sealed lineage differs from original native construction",
            ));
        }
        let report = self
            .store
            .object(&handle.validation_object_id)?
            .ok_or_else(invalid)?;
        if report.kind() != "wow.graph.lineage.validation" || report.schema_version() != 1 {
            return Err(invalid());
        }
        // The exact expected report identity is deterministic from this native
        // snapshot/object pair; a merely present 'valid' string cannot attest it.
        #[derive(Serialize)]
        struct Report<'a> {
            schema: &'static str,
            snapshot_id: &'a str,
            object_id: &'a ObjectId,
            status: &'static str,
            scope: &'static str,
        }
        let expected_report = PendingObject::from_json(
            "wow.graph.lineage.validation",
            1,
            &Report {
                schema: "wow-graph/lineage-candidate-validation/1",
                snapshot_id: &handle.snapshot_id,
                object_id: &handle.object_id,
                status: "valid",
                scope: self.profile.validation_scope,
            },
            self.store.configuration().limits(),
        )?;
        if expected_report.object_id() != &handle.validation_object_id {
            return Err(invalid());
        }
        Ok(record)
    }
    pub fn require_roots(&self, handle: &SealedLineageHandle) -> GraphResult<()> {
        let p = path(&handle.publication_id)?;
        for (name, id) in [
            (SEALED, &handle.object_id),
            (VALIDATION, &handle.validation_object_id),
        ] {
            if self
                .store
                .catalog_entry(&catalog(name)?, &p)?
                .is_none_or(|entry| entry.object_id() != id)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn read_sealed(
        &self,
        handle: &SealedLineageHandle,
        expected: &PreparedLineageSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<Box<[u8]>> {
        self.validate_sealed(handle, expected, stop)?;
        let object = self.store.object(&handle.object_id)?.ok_or_else(invalid)?;
        checkpoint(stop)?;
        Ok(object.canonical_json().into())
    }
    pub fn fence_roots(&self, handle: &SealedLineageHandle) -> GraphResult<WriteBatch> {
        self.require_roots(handle)?;
        let p = path(&handle.publication_id)?;
        let mut batch = WriteBatch::new();
        for (name, id) in [
            (SEALED, &handle.object_id),
            (VALIDATION, &handle.validation_object_id),
        ] {
            batch.add_catalog_mutation(CatalogMutation::set(
                catalog(name)?,
                p.clone(),
                CatalogExpectation::Exact(id.clone()),
                id.clone(),
            ))?;
        }
        Ok(batch)
    }
    pub fn plan_retire(&self, handle: &SealedLineageHandle) -> GraphResult<WriteBatch> {
        self.require_roots(handle)?;
        let p = path(&handle.publication_id)?;
        let mut batch = WriteBatch::new();
        for (name, id) in [
            (SEALED, &handle.object_id),
            (VALIDATION, &handle.validation_object_id),
        ] {
            batch.add_catalog_mutation(CatalogMutation::delete(
                catalog(name)?,
                p.clone(),
                id.clone(),
            ))?;
        }
        Ok(batch)
    }
    /// A failed target can be discarded only while no sealed/validation root
    /// exists. CAS removal preserves any concurrently changed inactive object.
    pub fn plan_abandon(&self, publication_id: &str) -> GraphResult<WriteBatch> {
        let p = path(publication_id)?;
        for name in [SEALED, VALIDATION] {
            if self.store.catalog_entry(&catalog(name)?, &p)?.is_some() {
                return Err(invalid());
            }
        }
        let mut batch = WriteBatch::new();
        if let Some(entry) = self.store.catalog_entry(&catalog(INACTIVE)?, &p)? {
            batch.add_catalog_mutation(CatalogMutation::delete(
                catalog(INACTIVE)?,
                p,
                entry.object_id().clone(),
            ))?;
        }
        Ok(batch)
    }
    fn read_record(&self, id: &ObjectId, stop: &AtomicBool) -> GraphResult<SnapshotRecord> {
        checkpoint(stop)?;
        let object = self.store.object(id)?.ok_or_else(invalid)?;
        if object.kind() != LINEAGE_SNAPSHOT_KIND || object.schema_version() != 1 {
            return Err(invalid());
        }
        let record: SnapshotRecord = object.decode()?;
        record.validate_identity(stop)?;
        if record.profile_id.as_ref() != self.profile.profile {
            return Err(invalid());
        }
        Ok(record)
    }
}
fn path(value: &str) -> GraphResult<CatalogPath> {
    Ok(CatalogPath::new(value)?)
}
fn catalog(value: &str) -> GraphResult<CatalogName> {
    Ok(CatalogName::new(value)?)
}
