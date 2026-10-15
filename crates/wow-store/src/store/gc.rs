use super::{
    OperationCommitBoundary, Store, apply_write_batch, complete_prepared_operation_in_transaction,
    epoch_to_i64, read_object, read_operation, validate_batch,
};
use crate::{
    GarbageCollectionOperationOutcome, GarbageCollectionReceipt, LogicalEpoch, ObjectId,
    OperationId, OperationRecord, OperationState, PendingObject, RequestDigest, StoreError,
    StoreErrorCode, StoreResult, WriteBatch,
};
use rusqlite::{Connection, TransactionBehavior, params};

const GC_OPERATION_RECEIPT_KIND: &str = "wow-store.garbage-collection-receipt";
const GC_OPERATION_RECEIPT_SCHEMA_VERSION: u32 = 1;

impl Store {
    /// Collect eligible objects and atomically persist the exact Store receipt
    /// under this operation ID and request digest.
    ///
    /// Consumers reserve a deterministic inner GC operation ID distinct from
    /// their own operation ID. A fresh operation row, deletions, receipt object,
    /// and completion commit together; a preprepared inner ID is refused.
    pub fn collect_garbage_with_operation(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> StoreResult<GarbageCollectionOperationOutcome> {
        self.collect_garbage_with_operation_observed(
            operation_id,
            request_digest,
            now,
            max_deletes,
            |_| Ok(()),
        )
    }

    /// Existing nonoperation GC preserves its original behavior and receipt.
    pub fn collect_garbage(
        &mut self,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> StoreResult<GarbageCollectionReceipt> {
        let limits = self.configuration.limits();
        let now = validate_gc_request(limits, now, max_deletes)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let receipt = collect_in_transaction(&transaction, now, max_deletes)?;
        transaction.commit().map_err(StoreError::database)?;
        Ok(receipt)
    }

    #[cfg(test)]
    pub(super) fn collect_garbage_with_operation_boundary<F>(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
        now: LogicalEpoch,
        max_deletes: u32,
        observe: F,
    ) -> StoreResult<GarbageCollectionOperationOutcome>
    where
        F: FnMut(OperationCommitBoundary) -> StoreResult<()>,
    {
        self.collect_garbage_with_operation_observed(
            operation_id,
            request_digest,
            now,
            max_deletes,
            observe,
        )
    }

    fn collect_garbage_with_operation_observed<F>(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
        now: LogicalEpoch,
        max_deletes: u32,
        mut observe: F,
    ) -> StoreResult<GarbageCollectionOperationOutcome>
    where
        F: FnMut(OperationCommitBoundary) -> StoreResult<()>,
    {
        let limits = self.configuration.limits();
        let now = validate_gc_request(limits, now, max_deletes)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;

        if let Some(record) = read_operation(&transaction, operation_id)? {
            if record.request_digest() != request_digest {
                return Err(StoreError::new(
                    StoreErrorCode::OperationConflict,
                    "GC operation ID is bound to another request digest",
                ));
            }
            match record.state() {
                OperationState::Completed => {
                    let receipt = read_gc_operation_receipt(
                        &transaction,
                        &record,
                        request_digest,
                        limits.max_object_bytes,
                    )?;
                    transaction.commit().map_err(StoreError::database)?;
                    return Ok(GarbageCollectionOperationOutcome::new(receipt, true));
                }
                OperationState::Prepared => {
                    return Err(StoreError::new(
                        StoreErrorCode::OutcomeUnknown,
                        "preprepared GC operation has no atomic receipt; refusing deletion",
                    ));
                }
                OperationState::OutcomeUnknown => {
                    return Err(StoreError::new(
                        StoreErrorCode::OutcomeUnknown,
                        "GC operation outcome is unknown and cannot be replayed",
                    ));
                }
                OperationState::NoEffect | OperationState::Failed => {
                    return Err(StoreError::new(
                        StoreErrorCode::OperationStateInvalid,
                        "GC operation has a recorded noncompleted outcome",
                    ));
                }
            }
        }

        let inserted = transaction
            .execute(
                "INSERT INTO store_operations(
                     operation_id, request_digest, state, result_object_id
                 ) VALUES (?1, ?2, 'prepared', NULL)",
                params![operation_id.as_str(), request_digest.as_str()],
            )
            .map_err(StoreError::database)?;
        if inserted != 1 {
            return Err(StoreError::new(
                StoreErrorCode::OperationStateInvalid,
                "GC operation preparation did not insert exactly one row",
            ));
        }

        let receipt = collect_in_transaction(&transaction, now, max_deletes)?;
        let result_object = PendingObject::from_json(
            GC_OPERATION_RECEIPT_KIND,
            GC_OPERATION_RECEIPT_SCHEMA_VERSION,
            &receipt,
            limits,
        )?;
        let result_object_id = result_object.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(result_object)?;
        if batch.objects().len() > limits.max_batch_objects as usize {
            return Err(StoreError::new(
                StoreErrorCode::BatchTooLarge,
                "GC receipt exceeds Store batch limits",
            ));
        }
        validate_batch(&batch, limits.max_object_bytes)?;
        let _ = apply_write_batch(&transaction, &batch, limits.max_object_bytes)?;
        let _ = complete_prepared_operation_in_transaction(
            &transaction,
            operation_id,
            request_digest,
            &result_object_id,
            limits.max_object_bytes,
        )?;
        observe(OperationCommitBoundary::BeforeCommit)?;
        transaction.commit().map_err(StoreError::database)?;
        observe(OperationCommitBoundary::AfterCommit)?;
        Ok(GarbageCollectionOperationOutcome::new(receipt, false))
    }
}

fn validate_gc_request(
    limits: crate::StoreLimits,
    now: LogicalEpoch,
    max_deletes: u32,
) -> StoreResult<i64> {
    if max_deletes == 0 || max_deletes > limits.max_gc_deletes {
        return Err(StoreError::new(
            StoreErrorCode::BudgetExceeded,
            "garbage collection delete budget is invalid",
        ));
    }
    epoch_to_i64(now)
}

fn collect_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    now: i64,
    max_deletes: u32,
) -> StoreResult<GarbageCollectionReceipt> {
    let expired = transaction
        .execute("DELETE FROM store_leases WHERE expires_after <= ?1", [now])
        .map_err(StoreError::database)?;
    let query_limit = i64::from(max_deletes) + 1;
    let mut statement = transaction
        .prepare(
            "SELECT object_id FROM store_objects AS object
             WHERE NOT EXISTS (
                 SELECT 1 FROM store_catalog AS catalog
                 WHERE catalog.object_id = object.object_id
             )
             AND NOT EXISTS (
                 SELECT 1 FROM store_operations AS operation
                 WHERE operation.result_object_id = object.object_id
             )
             AND NOT EXISTS (
                 SELECT 1 FROM store_leases AS lease
                 WHERE lease.object_id = object.object_id
             )
             ORDER BY object_id
             LIMIT ?1",
        )
        .map_err(StoreError::database)?;
    let candidates = statement
        .query_map([query_limit], |row| row.get::<_, String>(0))
        .map_err(StoreError::database)?
        .map(|row| row.map_err(StoreError::database).and_then(ObjectId::new))
        .collect::<StoreResult<Vec<_>>>()?;
    drop(statement);
    let more_available = candidates.len() > max_deletes as usize;
    let deleted = candidates
        .into_iter()
        .take(max_deletes as usize)
        .collect::<Vec<_>>();
    for object_id in &deleted {
        transaction
            .execute(
                "DELETE FROM store_objects WHERE object_id = ?1",
                [object_id.as_str()],
            )
            .map_err(StoreError::database)?;
    }
    let expired_leases_deleted = u64::try_from(expired).map_err(|_| {
        StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "deleted lease count was negative",
        )
    })?;
    Ok(GarbageCollectionReceipt::new(
        deleted,
        expired_leases_deleted,
        more_available,
    ))
}

fn read_gc_operation_receipt(
    connection: &Connection,
    operation: &OperationRecord,
    request_digest: &RequestDigest,
    max_object_bytes: u64,
) -> StoreResult<GarbageCollectionReceipt> {
    if operation.request_digest() != request_digest
        || operation.state() != OperationState::Completed
    {
        return Err(StoreError::new(
            StoreErrorCode::OperationStateInvalid,
            "GC operation has no completed receipt",
        ));
    }
    let result_object_id = operation.result_object_id().ok_or_else(|| {
        StoreError::new(
            StoreErrorCode::OperationStateInvalid,
            "completed GC operation has no receipt object",
        )
    })?;
    let object = read_object(connection, result_object_id, max_object_bytes)?.ok_or_else(|| {
        StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "completed GC receipt object is missing",
        )
    })?;
    if object.kind() != GC_OPERATION_RECEIPT_KIND
        || object.schema_version() != GC_OPERATION_RECEIPT_SCHEMA_VERSION
    {
        return Err(StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "completed GC operation points to another object type",
        ));
    }
    let receipt: GarbageCollectionReceipt = object.decode()?;
    let canonical = wow_core::canonical_json_bytes(&receipt).map_err(|_| {
        StoreError::new(
            StoreErrorCode::JsonInvalid,
            "GC receipt canonicalization failed",
        )
    })?;
    if canonical.as_slice() != object.canonical_json() {
        return Err(StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "stored GC receipt is not canonical",
        ));
    }
    Ok(receipt)
}
