//! Exact persistent generation holds in the selected v2 physical profile.
mod model;
use super::{ProjectStore, model::*, read::read_manifest};
use crate::{StoreError, StoreErrorCode, StoreResult};
pub use model::{RetentionRoot, RetentionRootId, RetentionRootKind};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::sync::atomic::AtomicBool;

const MAX_ROOTS: i64 = 1024;

impl ProjectStore {
    /// Hold a real generation. Exact retries preserve the original root;
    /// substituting its kind, holder or target under the same ID conflicts.
    pub fn put_retention_root(
        &mut self,
        root: &RetentionRoot,
        stop: &AtomicBool,
    ) -> StoreResult<RetentionRoot> {
        require_retention(&self.db.epoch)?;
        self.db.ensure_idle()?;
        checkpoint(stop)?;
        root.validate()?;
        if root.epoch_id() != self.db.epoch.epoch_id() {
            return Err(invalid());
        }
        read_manifest(&self.db.connection, root.generation_id(), &self.db.epoch)?;
        if let Some(existing) = read_root(&self.db.connection, root.root_id(), &self.db.epoch)? {
            checkpoint(stop)?;
            return exact_root(existing, root);
        }
        self.db.write_budget(65536)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        read_manifest(&tx, root.generation_id(), &self.db.epoch)?;
        if let Some(existing) = read_root(&tx, root.root_id(), &self.db.epoch)? {
            return exact_root(existing, root);
        }
        let count: i64 = tx
            .query_row("SELECT count(*) FROM retention_roots", [], |r| r.get(0))
            .map_err(StoreError::database)?;
        if count >= MAX_ROOTS {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        tx.execute(
            "INSERT INTO retention_roots(root_id,generation_id,record) VALUES(?1,?2,?3)",
            params![
                root.root_id().as_str(),
                root.generation_id().as_str(),
                encode(root, 65536)?
            ],
        )
        .map_err(StoreError::database)?;
        checkpoint(stop)?;
        let committed = tx.commit();
        // Failed commit plus failed drop-time rollback may leave a transaction
        // active. Same-connection readback would then observe pending writes.
        self.db.ensure_idle()?;
        match read_root(&self.db.connection, root.root_id(), &self.db.epoch) {
            Ok(Some(actual)) if actual == *root => Ok(actual),
            Ok(_) if committed.is_ok() => Err(invalid()),
            _ => Err(failure(StoreErrorCode::OutcomeUnknown)),
        }
    }

    /// Remove only the exact reviewed hold. Missing roots are an idempotent
    /// false result; a changed digest never authorizes removing another hold.
    pub fn remove_retention_root(
        &mut self,
        id: &RetentionRootId,
        expected_digest: &str,
        stop: &AtomicBool,
    ) -> StoreResult<bool> {
        require_retention(&self.db.epoch)?;
        self.db.ensure_idle()?;
        checkpoint(stop)?;
        let Some(existing) = read_root(&self.db.connection, id, &self.db.epoch)? else {
            return Ok(false);
        };
        if existing.pin_digest() != expected_digest {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        self.db.write_budget(65536)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let actual = read_root(&tx, id, &self.db.epoch)?.ok_or_else(invalid)?;
        if actual != existing {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        let changed = tx
            .execute(
                "DELETE FROM retention_roots WHERE root_id=?1 AND record=?2",
                params![id.as_str(), encode(&existing, 65536)?],
            )
            .map_err(StoreError::database)?;
        if changed != 1 {
            return Err(invalid());
        }
        checkpoint(stop)?;
        let committed = tx.commit();
        self.db.ensure_idle()?;
        match read_root(&self.db.connection, id, &self.db.epoch) {
            Ok(None) => Ok(true),
            Ok(_) if committed.is_ok() => Err(invalid()),
            _ => Err(failure(StoreErrorCode::OutcomeUnknown)),
        }
    }

    /// Canonical ordered, bounded inventory from SQL authority. This validates
    /// root identity and target manifests; it is not a domain integrity report.
    pub fn retention_roots(&self, stop: &AtomicBool) -> StoreResult<Vec<RetentionRoot>> {
        require_retention(&self.db.epoch)?;
        self.db.ensure_idle()?;
        checkpoint(stop)?;
        let mut statement = self.db.connection.prepare("SELECT CASE WHEN length(root_id)<=256 THEN root_id END FROM retention_roots ORDER BY root_id LIMIT 1025").map_err(StoreError::database)?;
        let rows = statement
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(StoreError::database)?;
        let mut result = Vec::new();
        for id in rows {
            checkpoint(stop)?;
            if result.len() >= MAX_ROOTS as usize {
                return Err(failure(StoreErrorCode::BudgetExceeded));
            }
            let id = RetentionRootId::new(id.map_err(StoreError::database)?)?;
            result.push(read_root(&self.db.connection, &id, &self.db.epoch)?.ok_or_else(invalid)?);
        }
        checkpoint(stop)?;
        Ok(result)
    }
}

fn require_retention(epoch: &EpochManifest) -> StoreResult<()> {
    if !matches!(
        epoch.physical_profile(),
        RETAINED_PHYSICAL_PROFILE | GC_PHYSICAL_PROFILE
    ) {
        return Err(failure(StoreErrorCode::ConfigurationInvalid));
    }
    Ok(())
}
fn exact_root(actual: RetentionRoot, expected: &RetentionRoot) -> StoreResult<RetentionRoot> {
    if &actual != expected {
        return Err(failure(StoreErrorCode::OperationConflict));
    }
    Ok(actual)
}
fn read_root(
    c: &Connection,
    id: &RetentionRootId,
    epoch: &EpochManifest,
) -> StoreResult<Option<RetentionRoot>> {
    let raw: Option<(String, Vec<u8>)> = c.query_row("SELECT CASE WHEN length(generation_id)<=128 THEN generation_id END,CASE WHEN length(record)<=65536 THEN record END FROM retention_roots WHERE root_id=?1",
        [id.as_str()], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(StoreError::database)?;
    let Some((generation, bytes)) = raw else {
        return Ok(None);
    };
    let root: RetentionRoot = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    root.validate()?;
    if root.root_id() != id
        || root.epoch_id() != epoch.epoch_id()
        || root.generation_id().as_str() != generation
        || encode(&root, 65536)? != bytes
    {
        return Err(invalid());
    }
    read_manifest(c, root.generation_id(), epoch)?;
    Ok(Some(root))
}
