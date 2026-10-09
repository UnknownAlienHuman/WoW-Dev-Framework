//! Bounded, exact-root SQL collection for self-contained partition records.
mod inventory;
mod model;
mod policy;
use super::{ProjectStore, database::Lifetime, model::*, release::require_release};
use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
pub use model::ProjectGcPolicy;
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    rc::{Rc, Weak},
    sync::atomic::AtomicBool,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGcReport {
    schema: String,
    epoch_id: EpochId,
    policy_digest: String,
    state_digest: String,
    lease_revision: u64,
    writer_changes: u64,
    data_version: i64,
    protected_generations: BTreeSet<StoreGenerationId>,
    delete_generations: Vec<StoreGenerationId>,
    delete_versions: Vec<PartitionVersionId>,
    payload_bytes: u64,
}
impl ProjectGcReport {
    pub fn protected_generations(&self) -> &BTreeSet<StoreGenerationId> {
        &self.protected_generations
    }
    pub fn delete_generations(&self) -> &[StoreGenerationId] {
        &self.delete_generations
    }
    pub fn delete_versions(&self) -> &[PartitionVersionId] {
        &self.delete_versions
    }
    pub fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }
    pub fn canonical_bytes(&self) -> StoreResult<Vec<u8>> {
        self.validate()?;
        encode(self, 2 * 1024 * 1024)
    }
    fn validate(&self) -> StoreResult<()> {
        if self.schema != "wow-store/project-gc-plan/1"
            || !hashed(&self.policy_digest, "project-gc-policy")
            || !hashed(&self.state_digest, "project-gc-state")
            || self.protected_generations.len() > MAX_GENERATIONS as usize
            || self.delete_generations.len() > MAX_GENERATIONS as usize
            || self.delete_versions.len() > MAX_VERSIONS as usize
            || self.payload_bytes > MAX_GENERATION_BYTES as u64
            || self.data_version < 0
            || self.lease_revision == u64::MAX
            || self.delete_generations.windows(2).any(|w| w[0] >= w[1])
            || self.delete_versions.windows(2).any(|w| w[0] >= w[1])
            || self
                .delete_generations
                .iter()
                .any(|id| self.protected_generations.contains(id))
        {
            return Err(invalid());
        }
        Ok(())
    }
}
/// A compiled owner capability; serialized reports cannot authorize deletion.
#[derive(Clone)]
pub struct ProjectGcPlan {
    report: ProjectGcReport,
    policy: ProjectGcPolicy,
    owner: Weak<Lifetime>,
}
impl ProjectGcPlan {
    pub fn report(&self) -> &ProjectGcReport {
        &self.report
    }
    pub fn request_digest(&self) -> StoreResult<String> {
        Ok(digest("project-gc-plan", &self.report.canonical_bytes()?))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGcReceipt {
    schema: String,
    operation_id: OperationId,
    request_digest: String,
    report: ProjectGcReport,
    receipt_digest: String,
}
impl ProjectGcReceipt {
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn report(&self) -> &ProjectGcReport {
        &self.report
    }
    pub fn receipt_digest(&self) -> &str {
        &self.receipt_digest
    }
    pub(super) fn canonical_bytes(&self) -> StoreResult<Vec<u8>> {
        encode(self, 2 * 1024 * 1024)
    }
    fn new(id: OperationId, report: ProjectGcReport) -> StoreResult<Self> {
        let request_digest = digest("project-gc-plan", &report.canonical_bytes()?);
        let schema = "wow-store/project-gc-receipt/1";
        let receipt_digest = digest(
            "project-gc-receipt",
            &encode(&(schema, &id, &request_digest, &report), 2 * 1024 * 1024)?,
        );
        Ok(Self {
            schema: schema.into(),
            operation_id: id,
            request_digest,
            report,
            receipt_digest,
        })
    }
}

impl ProjectStore {
    pub fn plan_gc(
        &self,
        policy: &ProjectGcPolicy,
        stop: &AtomicBool,
    ) -> StoreResult<ProjectGcPlan> {
        require_release(&self.db.epoch)?;
        self.db.ensure_idle()?;
        policy.validate()?;
        if self.gc_policy()?.as_ref() != Some(policy) {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        checkpoint(stop)?;
        let lease_revision = self.db.life.lease_revision.get();
        if lease_revision == u64::MAX {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        let writer_changes = self.db.connection.total_changes();
        let data_version = self.gc_data_version()?;
        let inventory = inventory::collect(self, policy, stop)?;
        let delete_generations: Vec<_> = inventory
            .generations
            .keys()
            .filter(|id| !inventory.protected.contains(*id))
            .take(policy.max_generations())
            .cloned()
            .collect();
        let selected: BTreeSet<_> = delete_generations.iter().cloned().collect();
        let mut retained_versions = inventory.operation_versions;
        for (id, manifest) in &inventory.generations {
            if !selected.contains(id) {
                retained_versions.extend(manifest.members.iter().map(|m| m.version.clone()));
            }
        }
        let mut delete_versions = Vec::new();
        let mut payload_bytes = 0u64;
        for (id, member) in &inventory.versions {
            checkpoint(stop)?;
            if retained_versions.contains(id) {
                continue;
            }
            let sum = payload_bytes
                .checked_add(member.byte_length as u64)
                .ok_or_else(invalid)?;
            if delete_versions.len() < policy.max_partition_versions()
                && sum <= policy.max_payload_bytes()
            {
                delete_versions.push(id.clone());
                payload_bytes = sum;
            }
        }
        if data_version != self.gc_data_version()?
            || writer_changes != self.db.connection.total_changes()
            || lease_revision != self.db.life.lease_revision.get()
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        let report = ProjectGcReport {
            schema: "wow-store/project-gc-plan/1".into(),
            epoch_id: self.db.epoch.epoch_id().clone(),
            policy_digest: policy.digest()?,
            state_digest: inventory.state_digest,
            lease_revision,
            writer_changes,
            data_version,
            protected_generations: inventory.protected,
            delete_generations,
            delete_versions,
            payload_bytes,
        };
        report.canonical_bytes()?;
        Ok(ProjectGcPlan {
            report,
            policy: policy.clone(),
            owner: Rc::downgrade(&self.db.life),
        })
    }

    /// Execute exactly the reviewed bounded batch, with its receipt in the same
    /// commit. Cancellation rolls back; response loss reconciles by exact ID.
    pub fn execute_gc(
        &mut self,
        plan: &ProjectGcPlan,
        id: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<ProjectGcReceipt> {
        require_release(&self.db.epoch)?;
        self.db.ensure_idle()?;
        checkpoint(stop)?;
        let request_digest = plan.request_digest()?;
        if self.operation(id)?.is_some() {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        if let Some(receipt) = self.reconcile_gc(id)? {
            if receipt.request_digest != request_digest {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            return Ok(receipt);
        }
        if !plan
            .owner
            .upgrade()
            .is_some_and(|owner| Rc::ptr_eq(&owner, &self.db.life))
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        let actual = self.plan_gc(&plan.policy, stop)?;
        if actual.report != plan.report {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        let receipt = ProjectGcReceipt::new(id.clone(), plan.report.clone())?;
        self.db.write_budget(2 * 1024 * 1024)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let locked_version: i64 = tx
            .query_row("PRAGMA data_version", [], |r| r.get(0))
            .map_err(StoreError::database)?;
        if locked_version != plan.report.data_version {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        let count: i64 = tx
            .query_row("SELECT count(*) FROM gc_operations", [], |r| r.get(0))
            .map_err(StoreError::database)?;
        if count >= MAX_GENERATIONS * 4 {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        for generation in &plan.report.delete_generations {
            checkpoint(stop)?;
            // FK order is deliberate. Current, explicit pins, operations and
            // process-local readers were protected by the exact recheck above.
            tx.execute(
                "DELETE FROM publication_history WHERE generation_id=?1",
                [generation.as_str()],
            )
            .map_err(StoreError::database)?;
            tx.execute(
                "DELETE FROM validations WHERE generation_id=?1",
                [generation.as_str()],
            )
            .map_err(StoreError::database)?;
            tx.execute(
                "DELETE FROM membership WHERE generation_id=?1",
                [generation.as_str()],
            )
            .map_err(StoreError::database)?;
            if tx
                .execute(
                    "DELETE FROM generations WHERE generation_id=?1",
                    [generation.as_str()],
                )
                .map_err(StoreError::database)?
                != 1
            {
                return Err(invalid());
            }
        }
        for version in &plan.report.delete_versions {
            checkpoint(stop)?;
            if tx
                .execute(
                    "DELETE FROM partition_versions WHERE version=?1",
                    [version.as_str()],
                )
                .map_err(StoreError::database)?
                != 1
            {
                return Err(invalid());
            }
        }
        let failed: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
                [],
                |r| r.get(0),
            )
            .map_err(StoreError::database)?;
        if failed {
            return Err(invalid());
        }
        super::read::read_current(&tx, &self.db.epoch)?;
        tx.execute(
            "INSERT INTO gc_operations(operation_id,request_digest,record) VALUES(?1,?2,?3)",
            params![
                id.as_str(),
                request_digest,
                encode(&receipt, 2 * 1024 * 1024)?
            ],
        )
        .map_err(StoreError::database)?;
        checkpoint(stop)?;
        let committed = tx.commit();
        self.db.ensure_idle()?;
        match self.reconcile_gc(id) {
            Ok(Some(actual)) if actual == receipt => Ok(actual),
            Ok(_) if committed.is_ok() => Err(invalid()),
            _ => Err(failure(StoreErrorCode::OutcomeUnknown)),
        }
    }

    pub fn reconcile_gc(&self, id: &OperationId) -> StoreResult<Option<ProjectGcReceipt>> {
        require_release(&self.db.epoch)?;
        self.db.ensure_idle()?;
        OperationId::new(id.as_str())?;
        let raw: Option<(String,Vec<u8>)> = self.db.connection.query_row("SELECT CASE WHEN length(request_digest)<=128 THEN request_digest END,CASE WHEN length(record)<=2097152 THEN record END FROM gc_operations WHERE operation_id=?1",
            [id.as_str()], |r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(StoreError::database)?;
        let Some((request_digest, bytes)) = raw else {
            return Ok(None);
        };
        let receipt: ProjectGcReceipt = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        let expected = ProjectGcReceipt::new(id.clone(), receipt.report.clone())?;
        if receipt != expected
            || receipt.request_digest != request_digest
            || receipt.report.epoch_id != *self.db.epoch.epoch_id()
            || encode(&receipt, 2 * 1024 * 1024)? != bytes
        {
            return Err(invalid());
        }
        Ok(Some(receipt))
    }
    fn gc_data_version(&self) -> StoreResult<i64> {
        self.db
            .connection
            .query_row("PRAGMA data_version", [], |r| r.get(0))
            .map_err(StoreError::database)
    }
}
