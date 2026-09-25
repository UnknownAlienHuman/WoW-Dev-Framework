use super::{ProjectStore, ValidatedRead, model::*, read::*};
use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
use rusqlite::{TransactionBehavior, params};
use std::sync::atomic::AtomicBool;

impl ProjectStore {
    /// Seal immutable partition versions and publish complete inactive membership.
    /// Cancellation can leave sealed inert versions, never a changed current.
    pub fn prepare(
        &mut self,
        request: &PublicationRequest,
        stop: &AtomicBool,
    ) -> StoreResult<PublicationOperation> {
        request.manifest.validate(&self.db.epoch)?;
        checkpoint(stop)?;
        if let Some(op) = self.operation(&request.operation_id)? {
            ensure_request(&op, request)?;
            if op.state != PublicationState::Prepared {
                return Ok(op);
            }
        }
        require_base(&self.db.connection, &request.manifest, &self.db.epoch)?;
        self.db.write_budget(256 * 1024)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        require_base(&tx, &request.manifest, &self.db.epoch)?;
        if read_operation(&tx, &request.operation_id, &self.db.epoch)?.is_none() {
            let count: i64 = tx
                .query_row("SELECT count(*) FROM operations", [], |r| r.get(0))
                .map_err(StoreError::database)?;
            if count >= MAX_GENERATIONS * 4 {
                return Err(failure(StoreErrorCode::BudgetExceeded));
            }
            let op = PublicationOperation {
                operation_id: request.operation_id.clone(),
                request_digest: request.digest.clone(),
                generation_id: request.manifest.generation_id.clone(),
                state: PublicationState::Prepared,
                validation_id: None,
                activation: None,
            };
            tx.execute("INSERT INTO operations(operation_id,request_digest,manifest,record) VALUES(?1,?2,?3,?4)",
                params![request.operation_id.as_str(),request.digest,encode(&request.manifest,256 * 1024)?,encode(&op,65536)?])
                .map_err(StoreError::database)?;
        }
        checkpoint(stop)?;
        tx.commit()
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        for (record, member) in request.records.iter().zip(&request.manifest.members) {
            checkpoint(stop)?;
            // Probe reuse before reserving write space. Equivalent sealed data is
            // verified and not rewritten, even following a response loss.
            let exists: bool = self
                .db
                .connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM partition_versions WHERE version=?1)",
                    [record.version.as_str()],
                    |r| r.get(0),
                )
                .map_err(StoreError::database)?;
            if exists {
                read_partition(&self.db.connection, member)?;
                continue;
            }
            self.db.write_budget(record.bytes.len())?;
            let tx = self
                .db
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(StoreError::database)?;
            let count: i64 = tx
                .query_row("SELECT count(*) FROM partition_versions", [], |r| r.get(0))
                .map_err(StoreError::database)?;
            if count >= MAX_VERSIONS {
                return Err(failure(StoreErrorCode::BudgetExceeded));
            }
            tx.execute("INSERT INTO partition_versions(version,logical_key,schema_id,byte_length,payload) VALUES(?1,?2,?3,?4,?5)",
                params![record.version.as_str(),record.key,record.schema,record.bytes.len() as i64,record.bytes]).map_err(StoreError::database)?;
            // Recompute the logical identity from read-back bytes inside the seal transaction.
            read_partition(&tx, member)?;
            checkpoint(stop)?;
            tx.commit()
                .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        }
        checkpoint(stop)?;
        self.db.write_budget(512 * 1024)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        require_base(&tx, &request.manifest, &self.db.epoch)?;
        let old =
            read_operation(&tx, &request.operation_id, &self.db.epoch)?.ok_or_else(invalid)?;
        ensure_request(&old, request)?;
        let existing: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM generations WHERE generation_id=?1)",
                [request.manifest.generation_id.as_str()],
                |r| r.get(0),
            )
            .map_err(StoreError::database)?;
        if existing {
            if read_manifest(&tx, &request.manifest.generation_id, &self.db.epoch)?
                != request.manifest
            {
                return Err(invalid());
            }
        } else {
            let count: i64 = tx
                .query_row("SELECT count(*) FROM generations", [], |r| r.get(0))
                .map_err(StoreError::database)?;
            if count >= MAX_GENERATIONS {
                return Err(failure(StoreErrorCode::BudgetExceeded));
            }
            tx.execute(
                "INSERT INTO generations(generation_id,manifest) VALUES(?1,?2)",
                params![
                    request.manifest.generation_id.as_str(),
                    encode(&request.manifest, 256 * 1024)?
                ],
            )
            .map_err(StoreError::database)?;
            for member in &request.manifest.members {
                checkpoint(stop)?;
                tx.execute(
                    "INSERT INTO membership(generation_id,logical_key,version) VALUES(?1,?2,?3)",
                    params![
                        request.manifest.generation_id.as_str(),
                        member.key,
                        member.version.as_str()
                    ],
                )
                .map_err(StoreError::database)?;
            }
        }
        let mut op = old.clone();
        op.state = PublicationState::PublishedInactive;
        save_operation(&tx, &old, &op)?;
        checkpoint(stop)?;
        let committed = tx.commit();
        self.observe_commit(&op, committed)
    }

    /// Persist successful structural/owner read-back separately from activation.
    /// This method cannot be called with a serialized success flag or report.
    pub fn validate_inactive(
        &mut self,
        id: &OperationId,
        request_digest: &str,
        validated: ValidatedRead,
        stop: &AtomicBool,
    ) -> StoreResult<PublicationOperation> {
        checkpoint(stop)?;
        if validated.epoch != self.db.epoch.epoch_id {
            return Err(invalid());
        }
        self.db.write_budget(65536)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let old = read_operation(&tx, id, &self.db.epoch)?.ok_or_else(invalid)?;
        if old.request_digest != request_digest
            || old.generation_id != validated.validation.generation_id
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        if matches!(
            old.state,
            PublicationState::Activated | PublicationState::ValidatedInactive
        ) {
            if old.validation_id.as_ref() != Some(&validated.validation.validation_id) {
                return Err(invalid());
            }
            return Ok(old);
        }
        if old.state != PublicationState::PublishedInactive {
            return Err(failure(StoreErrorCode::OperationStateInvalid));
        }
        let manifest = read_manifest(&tx, &old.generation_id, &self.db.epoch)?;
        let expected = ValidationRecord::new(&manifest, self.db.epoch.catalog.checks().clone())?;
        if validated.validation != expected {
            return Err(invalid());
        }
        tx.execute("INSERT OR IGNORE INTO validations(validation_id,generation_id,record) VALUES(?1,?2,?3)",
            params![expected.validation_id.as_str(),manifest.generation_id.as_str(),encode(&expected,65536)?]).map_err(StoreError::database)?;
        read_validation(&tx, &expected.validation_id, &manifest, &self.db.epoch)?;
        let mut op = old.clone();
        op.state = PublicationState::ValidatedInactive;
        op.validation_id = Some(expected.validation_id);
        save_operation(&tx, &old, &op)?;
        checkpoint(stop)?;
        let committed = tx.commit();
        self.observe_commit(&op, committed)
    }

    /// CAS the sole current record and activation history/receipt in one commit.
    /// A committed effect is returned even if cancellation arrived during commit.
    pub fn activate(
        &mut self,
        id: &OperationId,
        request_digest: &str,
        stop: &AtomicBool,
    ) -> StoreResult<PublicationOperation> {
        if let Some(op) = self.operation(id)? {
            if op.request_digest != request_digest {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            if op.state == PublicationState::Activated {
                return Ok(op);
            }
        }
        checkpoint(stop)?;
        self.db.write_budget(65536)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let old = read_operation(&tx, id, &self.db.epoch)?.ok_or_else(invalid)?;
        if old.request_digest != request_digest {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        if old.state != PublicationState::ValidatedInactive {
            return Err(failure(StoreErrorCode::OperationStateInvalid));
        }
        let manifest = read_manifest(&tx, &old.generation_id, &self.db.epoch)?;
        require_base(&tx, &manifest, &self.db.epoch)?;
        let validation_id = old.validation_id.as_ref().ok_or_else(invalid)?;
        read_validation(&tx, validation_id, &manifest, &self.db.epoch)?;
        let current = CurrentPublication::new(&manifest, validation_id.clone())?;
        tx.execute("INSERT INTO publication_history(record_id,generation_id,validation_id,record) VALUES(?1,?2,?3,?4)",
            params![current.record_id.as_str(),current.generation_id.as_str(),current.validation_id.as_str(),encode(&current,65536)?]).map_err(StoreError::database)?;
        let changed = if let Some(base) = &manifest.expected_current {
            tx.execute(
                "UPDATE current_publication SET record_id=?1 WHERE id=1 AND record_id=?2",
                params![current.record_id.as_str(), base.as_str()],
            )
        } else {
            tx.execute(
                "INSERT OR IGNORE INTO current_publication(id,record_id) VALUES(1,?1)",
                [current.record_id.as_str()],
            )
        }
        .map_err(StoreError::database)?;
        if changed != 1 {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        let mut op = old.clone();
        op.state = PublicationState::Activated;
        op.activation = Some(current);
        save_operation(&tx, &old, &op)?;
        checkpoint(stop)?;
        let committed = tx.commit();
        self.observe_commit(&op, committed)
    }

    fn observe_commit(
        &self,
        expected: &PublicationOperation,
        committed: rusqlite::Result<()>,
    ) -> StoreResult<PublicationOperation> {
        match self.operation(&expected.operation_id) {
            Ok(Some(op)) if op == *expected => Ok(op),
            Ok(_) if committed.is_ok() => Err(invalid()),
            _ => Err(failure(StoreErrorCode::OutcomeUnknown)),
        }
    }

    /// Read-only recovery classification for one exact operation, not a retry,
    /// automatic rollback, destructive repair, or selection of last-known-good.
    pub fn reconcile(&self, id: &OperationId) -> StoreResult<Option<PublicationOperation>> {
        self.operation(id)
    }
}
fn ensure_request(op: &PublicationOperation, request: &PublicationRequest) -> StoreResult<()> {
    if op.request_digest != request.digest || op.generation_id != request.manifest.generation_id {
        Err(failure(StoreErrorCode::OperationConflict))
    } else {
        Ok(())
    }
}
