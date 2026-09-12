use std::{path::Path, time::Duration};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::{
    CatalogChange, CatalogEntry, CatalogExpectation, CatalogMutation, CatalogName, CatalogPath,
    CommitReceipt, GarbageCollectionReceipt, IntegrityReport, LeaseId, LeaseRecord, LogicalEpoch,
    LogicalManifest, ObjectId, ObjectRecord, OperationBegin, OperationId, OperationRecord,
    OperationState, RequestDigest, StoreConfiguration, StoreError, StoreErrorCode, StoreResult,
    WriteBatch,
};

const APPLICATION_ID: i64 = 0x5744_4631;
const USER_VERSION: i64 = 1;

/// One synchronous durable store. SQLite handles and physical schema stay private.
pub struct Store {
    connection: Connection,
    configuration: StoreConfiguration,
}

impl Store {
    pub fn open(path: impl AsRef<Path>, configuration: StoreConfiguration) -> StoreResult<Self> {
        let connection = Connection::open(path).map_err(StoreError::database)?;
        Self::from_connection(connection, configuration)
    }

    pub fn open_in_memory(configuration: StoreConfiguration) -> StoreResult<Self> {
        let connection = Connection::open_in_memory().map_err(StoreError::database)?;
        Self::from_connection(connection, configuration)
    }

    fn from_connection(
        connection: Connection,
        configuration: StoreConfiguration,
    ) -> StoreResult<Self> {
        configuration.limits().validate()?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(StoreError::database)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA trusted_schema = OFF;
                 PRAGMA synchronous = FULL;
                 CREATE TABLE IF NOT EXISTS store_meta (
                     key TEXT PRIMARY KEY NOT NULL,
                     value TEXT NOT NULL
                 ) STRICT;
                 CREATE TABLE IF NOT EXISTS store_objects (
                     object_id TEXT PRIMARY KEY NOT NULL,
                     kind TEXT NOT NULL,
                     schema_version INTEGER NOT NULL CHECK(schema_version > 0),
                     content_sha256 TEXT NOT NULL,
                     canonical_json BLOB NOT NULL
                 ) STRICT;
                 CREATE TABLE IF NOT EXISTS store_catalog (
                     catalog TEXT NOT NULL,
                     path TEXT NOT NULL,
                     object_id TEXT NOT NULL REFERENCES store_objects(object_id) ON DELETE RESTRICT,
                     PRIMARY KEY (catalog, path)
                 ) STRICT;
                 CREATE TABLE IF NOT EXISTS store_operations (
                     operation_id TEXT PRIMARY KEY NOT NULL,
                     request_digest TEXT NOT NULL,
                     state TEXT NOT NULL,
                     result_object_id TEXT REFERENCES store_objects(object_id) ON DELETE RESTRICT,
                     CHECK ((state = 'completed' AND result_object_id IS NOT NULL)
                         OR (state <> 'completed' AND result_object_id IS NULL))
                 ) STRICT;
                 CREATE TABLE IF NOT EXISTS store_leases (
                     lease_id TEXT PRIMARY KEY NOT NULL,
                     object_id TEXT NOT NULL REFERENCES store_objects(object_id) ON DELETE RESTRICT,
                     holder TEXT NOT NULL,
                     expires_after INTEGER NOT NULL CHECK(expires_after >= 0)
                 ) STRICT;
                 CREATE INDEX IF NOT EXISTS store_catalog_object_idx
                     ON store_catalog(object_id);
                 CREATE INDEX IF NOT EXISTS store_operation_result_idx
                     ON store_operations(result_object_id);
                 CREATE INDEX IF NOT EXISTS store_lease_object_idx
                     ON store_leases(object_id);
                 CREATE INDEX IF NOT EXISTS store_lease_expiry_idx
                     ON store_leases(expires_after);",
            )
            .map_err(StoreError::database)?;
        let application_id: i64 = connection
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .map_err(StoreError::database)?;
        if application_id == 0 {
            connection
                .pragma_update(None, "application_id", APPLICATION_ID)
                .map_err(StoreError::database)?;
        } else if application_id != APPLICATION_ID {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "database application identity is incompatible",
            ));
        }
        let user_version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(StoreError::database)?;
        if user_version == 0 {
            connection
                .pragma_update(None, "user_version", USER_VERSION)
                .map_err(StoreError::database)?;
        } else if user_version != USER_VERSION {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "database schema version is incompatible",
            ));
        }
        let existing: Option<String> = connection
            .query_row(
                "SELECT value FROM store_meta WHERE key = 'configuration_id'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::database)?;
        match existing {
            Some(value) if value != configuration.configuration_id() => {
                return Err(StoreError::new(
                    StoreErrorCode::ConfigurationInvalid,
                    "store configuration does not match the durable database",
                ));
            }
            Some(_) => {}
            None => {
                connection
                    .execute(
                        "INSERT INTO store_meta(key, value) VALUES ('configuration_id', ?1)",
                        [configuration.configuration_id()],
                    )
                    .map_err(StoreError::database)?;
            }
        }
        Ok(Self {
            connection,
            configuration,
        })
    }

    #[must_use]
    pub fn configuration(&self) -> &StoreConfiguration {
        &self.configuration
    }

    pub fn object(&self, object_id: &ObjectId) -> StoreResult<Option<ObjectRecord>> {
        read_object(&self.connection, object_id)
    }

    pub fn catalog_entry(
        &self,
        catalog: &CatalogName,
        path: &CatalogPath,
    ) -> StoreResult<Option<CatalogEntry>> {
        let object_id: Option<String> = self
            .connection
            .query_row(
                "SELECT object_id FROM store_catalog WHERE catalog = ?1 AND path = ?2",
                params![catalog.as_str(), path.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::database)?;
        object_id
            .map(|value| {
                Ok(CatalogEntry::new(
                    catalog.clone(),
                    path.clone(),
                    ObjectId::new(value)?,
                ))
            })
            .transpose()
    }

    pub fn commit(&mut self, batch: WriteBatch) -> StoreResult<CommitReceipt> {
        let limits = self.configuration.limits();
        if batch.objects().len() > limits.max_batch_objects as usize
            || batch.catalog_mutations().len() > limits.max_catalog_mutations as usize
        {
            return Err(StoreError::new(
                StoreErrorCode::BatchTooLarge,
                "write batch exceeds configured limits",
            ));
        }
        validate_batch(&batch, limits.max_object_bytes)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let mut inserted = Vec::new();
        let mut deduplicated = Vec::new();
        for object in batch.objects() {
            let changed = transaction
                .execute(
                    "INSERT OR IGNORE INTO store_objects(
                         object_id, kind, schema_version, content_sha256, canonical_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        object.object_id().as_str(),
                        object.kind(),
                        i64::from(object.schema_version()),
                        object.content_sha256(),
                        object.canonical_json(),
                    ],
                )
                .map_err(StoreError::database)?;
            if changed == 1 {
                inserted.push(object.object_id().clone());
            } else {
                let existing = read_object(&transaction, object.object_id())?.ok_or_else(|| {
                    StoreError::new(
                        StoreErrorCode::IntegrityViolation,
                        "deduplicated object disappeared inside transaction",
                    )
                })?;
                if existing.kind() != object.kind()
                    || existing.schema_version() != object.schema_version()
                    || existing.content_sha256() != object.content_sha256()
                    || existing.canonical_json() != object.canonical_json()
                {
                    return Err(StoreError::new(
                        StoreErrorCode::ObjectConflict,
                        "object id is already bound to different bytes",
                    ));
                }
                deduplicated.push(object.object_id().clone());
            }
        }
        let mut changes = Vec::new();
        for mutation in batch.catalog_mutations() {
            changes.push(apply_catalog_mutation(&transaction, mutation)?);
        }
        transaction.commit().map_err(StoreError::database)?;
        CommitReceipt::build(inserted, deduplicated, changes)
    }

    pub fn begin_operation(
        &mut self,
        operation_id: OperationId,
        request_digest: RequestDigest,
    ) -> StoreResult<OperationBegin> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let inserted = transaction
            .execute(
                "INSERT OR IGNORE INTO store_operations(
                     operation_id, request_digest, state, result_object_id
                 ) VALUES (?1, ?2, 'prepared', NULL)",
                params![operation_id.as_str(), request_digest.as_str()],
            )
            .map_err(StoreError::database)?
            == 1;
        let record = read_operation(&transaction, &operation_id)?.ok_or_else(|| {
            StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "operation disappeared inside transaction",
            )
        })?;
        if record.request_digest() != &request_digest {
            return Err(StoreError::new(
                StoreErrorCode::OperationConflict,
                "operation id is already bound to a different request digest",
            ));
        }
        transaction.commit().map_err(StoreError::database)?;
        Ok(if inserted {
            OperationBegin::Started(record)
        } else {
            OperationBegin::Replay(record)
        })
    }

    pub fn complete_operation(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
        result_object_id: &ObjectId,
    ) -> StoreResult<OperationRecord> {
        self.transition_operation(
            operation_id,
            request_digest,
            OperationState::Completed,
            Some(result_object_id),
        )
    }

    pub fn record_no_effect(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
    ) -> StoreResult<OperationRecord> {
        self.transition_operation(operation_id, request_digest, OperationState::NoEffect, None)
    }

    pub fn mark_outcome_unknown(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
    ) -> StoreResult<OperationRecord> {
        self.transition_operation(
            operation_id,
            request_digest,
            OperationState::OutcomeUnknown,
            None,
        )
    }

    pub fn record_failed(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
    ) -> StoreResult<OperationRecord> {
        self.transition_operation(operation_id, request_digest, OperationState::Failed, None)
    }

    fn transition_operation(
        &mut self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
        state: OperationState,
        result_object_id: Option<&ObjectId>,
    ) -> StoreResult<OperationRecord> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let current = read_operation(&transaction, operation_id)?.ok_or_else(|| {
            StoreError::new(
                StoreErrorCode::OperationStateInvalid,
                "operation was not prepared",
            )
        })?;
        if current.request_digest() != request_digest {
            return Err(StoreError::new(
                StoreErrorCode::OperationConflict,
                "operation request digest does not match",
            ));
        }
        if current.state() != OperationState::Prepared {
            let same_terminal =
                current.state() == state && current.result_object_id() == result_object_id;
            if same_terminal {
                transaction.commit().map_err(StoreError::database)?;
                return Ok(current);
            }
            return Err(StoreError::new(
                StoreErrorCode::OperationStateInvalid,
                "operation is already terminal",
            ));
        }
        if let Some(object_id) = result_object_id
            && read_object(&transaction, object_id)?.is_none()
        {
            return Err(StoreError::new(
                StoreErrorCode::ObjectMissing,
                "operation result object does not exist",
            ));
        }
        transaction
            .execute(
                "UPDATE store_operations
                 SET state = ?1, result_object_id = ?2
                 WHERE operation_id = ?3 AND request_digest = ?4 AND state = 'prepared'",
                params![
                    state.as_str(),
                    result_object_id.map(ObjectId::as_str),
                    operation_id.as_str(),
                    request_digest.as_str(),
                ],
            )
            .map_err(StoreError::database)?;
        let record = read_operation(&transaction, operation_id)?.ok_or_else(|| {
            StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "operation disappeared during transition",
            )
        })?;
        transaction.commit().map_err(StoreError::database)?;
        Ok(record)
    }

    pub fn acquire_lease(
        &mut self,
        lease_id: LeaseId,
        object_id: ObjectId,
        holder: impl Into<Box<str>>,
        expires_after: LogicalEpoch,
    ) -> StoreResult<LeaseRecord> {
        let holder = holder.into();
        let requested = LeaseRecord::new(
            lease_id.clone(),
            object_id.clone(),
            holder.clone(),
            expires_after,
        )?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        if read_object(&transaction, &object_id)?.is_none() {
            return Err(StoreError::new(
                StoreErrorCode::ObjectMissing,
                "lease object does not exist",
            ));
        }
        let inserted = transaction
            .execute(
                "INSERT OR IGNORE INTO store_leases(lease_id, object_id, holder, expires_after)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    lease_id.as_str(),
                    object_id.as_str(),
                    holder.as_ref(),
                    epoch_to_i64(expires_after)?,
                ],
            )
            .map_err(StoreError::database)?
            == 1;
        let current = read_lease(&transaction, &lease_id)?.ok_or_else(|| {
            StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "lease disappeared inside transaction",
            )
        })?;
        if !inserted && current != requested {
            return Err(StoreError::new(
                StoreErrorCode::LeaseConflict,
                "lease id is already bound to another lease",
            ));
        }
        transaction.commit().map_err(StoreError::database)?;
        Ok(current)
    }

    pub fn renew_lease(
        &mut self,
        lease_id: &LeaseId,
        holder: &str,
        expected_expiry: LogicalEpoch,
        new_expiry: LogicalEpoch,
    ) -> StoreResult<LeaseRecord> {
        if new_expiry.get() <= expected_expiry.get() {
            return Err(StoreError::new(
                StoreErrorCode::LeaseInvalid,
                "lease renewal must advance the logical expiry",
            ));
        }
        let changed = self
            .connection
            .execute(
                "UPDATE store_leases SET expires_after = ?1
                 WHERE lease_id = ?2 AND holder = ?3 AND expires_after = ?4",
                params![
                    epoch_to_i64(new_expiry)?,
                    lease_id.as_str(),
                    holder,
                    epoch_to_i64(expected_expiry)?,
                ],
            )
            .map_err(StoreError::database)?;
        if changed != 1 {
            return Err(StoreError::new(
                StoreErrorCode::LeaseConflict,
                "lease renewal guard does not match",
            ));
        }
        read_lease(&self.connection, lease_id)?.ok_or_else(|| {
            StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "renewed lease disappeared",
            )
        })
    }

    pub fn release_lease(&mut self, lease_id: &LeaseId, holder: &str) -> StoreResult<()> {
        let changed = self
            .connection
            .execute(
                "DELETE FROM store_leases WHERE lease_id = ?1 AND holder = ?2",
                params![lease_id.as_str(), holder],
            )
            .map_err(StoreError::database)?;
        if changed != 1 {
            return Err(StoreError::new(
                StoreErrorCode::LeaseConflict,
                "lease release guard does not match",
            ));
        }
        Ok(())
    }

    pub fn collect_garbage(
        &mut self,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> StoreResult<GarbageCollectionReceipt> {
        if max_deletes == 0 || max_deletes > self.configuration.limits().max_gc_deletes {
            return Err(StoreError::new(
                StoreErrorCode::BudgetExceeded,
                "garbage collection delete budget is invalid",
            ));
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let expired = transaction
            .execute(
                "DELETE FROM store_leases WHERE expires_after <= ?1",
                [epoch_to_i64(now)?],
            )
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
        transaction.commit().map_err(StoreError::database)?;
        Ok(GarbageCollectionReceipt::new(
            deleted,
            expired as u64,
            more_available,
        ))
    }

    pub fn validate_integrity(&self, max_objects: u32) -> StoreResult<IntegrityReport> {
        if max_objects == 0 || max_objects > self.configuration.limits().max_manifest_records {
            return Err(StoreError::new(
                StoreErrorCode::BudgetExceeded,
                "integrity object budget is invalid",
            ));
        }
        let sqlite: String = self
            .connection
            .query_row("PRAGMA integrity_check(1)", [], |row| row.get(0))
            .map_err(StoreError::database)?;
        if sqlite != "ok" {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "SQLite integrity validation failed",
            ));
        }
        let mut statement = self
            .connection
            .prepare(
                "SELECT object_id, kind, schema_version, content_sha256, canonical_json
                 FROM store_objects ORDER BY object_id LIMIT ?1",
            )
            .map_err(StoreError::database)?;
        let rows = statement
            .query_map([i64::from(max_objects) + 1], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                ))
            })
            .map_err(StoreError::database)?;
        let mut checked_objects = 0_u64;
        let mut complete = true;
        for row in rows {
            if checked_objects == u64::from(max_objects) {
                complete = false;
                break;
            }
            let (object_id, kind, schema_version, content_sha256, canonical_json) =
                row.map_err(StoreError::database)?;
            ObjectRecord::from_parts(
                ObjectId::new(object_id)?,
                kind.into(),
                u32::try_from(schema_version).map_err(|_| {
                    StoreError::new(
                        StoreErrorCode::IntegrityViolation,
                        "stored object schema version is invalid",
                    )
                })?,
                content_sha256.into(),
                canonical_json.into_boxed_slice(),
            )?;
            checked_objects += 1;
        }
        let catalog = table_count(&self.connection, "store_catalog")?;
        let operations = table_count(&self.connection, "store_operations")?;
        let leases = table_count(&self.connection, "store_leases")?;
        let foreign_keys: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .map_err(StoreError::database)?;
        if foreign_keys != 0 {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "stored references violate object ownership",
            ));
        }
        Ok(IntegrityReport::new(
            checked_objects,
            catalog,
            operations,
            leases,
            complete,
        ))
    }

    pub fn logical_manifest(&self) -> StoreResult<LogicalManifest> {
        let limit = self.configuration.limits().max_manifest_records;
        let objects = query_ids(
            &self.connection,
            "SELECT object_id FROM store_objects ORDER BY object_id",
            limit,
        )?;
        let catalog_entries = query_catalog(&self.connection, limit)?;
        let operations = query_operations(&self.connection, limit)?;
        let leases = query_leases(&self.connection, limit)?;
        LogicalManifest::build(
            self.configuration.configuration_id().into(),
            objects,
            catalog_entries,
            operations,
            leases,
        )
    }
}

fn validate_batch(batch: &WriteBatch, max_object_bytes: u64) -> StoreResult<()> {
    for object in batch.objects() {
        if object.canonical_json().len() as u64 > max_object_bytes {
            return Err(StoreError::new(
                StoreErrorCode::ObjectTooLarge,
                "batch object exceeds the configured byte budget",
            ));
        }
    }
    Ok(())
}

fn apply_catalog_mutation(
    transaction: &Transaction<'_>,
    mutation: &CatalogMutation,
) -> StoreResult<CatalogChange> {
    let current: Option<String> = transaction
        .query_row(
            "SELECT object_id FROM store_catalog WHERE catalog = ?1 AND path = ?2",
            params![mutation.catalog().as_str(), mutation.path().as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::database)?;
    let current = current.map(ObjectId::new).transpose()?;
    let matches = match mutation.expectation() {
        CatalogExpectation::Absent => current.is_none(),
        CatalogExpectation::Exact(expected) => current.as_ref() == Some(expected),
    };
    if !matches {
        return Err(StoreError::new(
            StoreErrorCode::CatalogConflict,
            "catalog compare-and-swap guard does not match",
        ));
    }
    if let Some(target) = mutation.target() {
        if read_object(transaction, target)?.is_none() {
            return Err(StoreError::new(
                StoreErrorCode::ObjectMissing,
                "catalog target object does not exist",
            ));
        }
        transaction
            .execute(
                "INSERT INTO store_catalog(catalog, path, object_id)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(catalog, path) DO UPDATE SET object_id = excluded.object_id",
                params![
                    mutation.catalog().as_str(),
                    mutation.path().as_str(),
                    target.as_str(),
                ],
            )
            .map_err(StoreError::database)?;
    } else {
        transaction
            .execute(
                "DELETE FROM store_catalog WHERE catalog = ?1 AND path = ?2",
                params![mutation.catalog().as_str(), mutation.path().as_str()],
            )
            .map_err(StoreError::database)?;
    }
    Ok(CatalogChange::new(
        mutation.catalog().clone(),
        mutation.path().clone(),
        current,
        mutation.target().cloned(),
    ))
}

fn read_object(connection: &Connection, object_id: &ObjectId) -> StoreResult<Option<ObjectRecord>> {
    let row = connection
        .query_row(
            "SELECT kind, schema_version, content_sha256, canonical_json
             FROM store_objects WHERE object_id = ?1",
            [object_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::database)?;
    row.map(|(kind, schema_version, content_sha256, canonical_json)| {
        ObjectRecord::from_parts(
            object_id.clone(),
            kind.into(),
            u32::try_from(schema_version).map_err(|_| {
                StoreError::new(
                    StoreErrorCode::IntegrityViolation,
                    "stored object schema version is invalid",
                )
            })?,
            content_sha256.into(),
            canonical_json.into_boxed_slice(),
        )
    })
    .transpose()
}

fn read_operation(
    connection: &Connection,
    operation_id: &OperationId,
) -> StoreResult<Option<OperationRecord>> {
    let row = connection
        .query_row(
            "SELECT request_digest, state, result_object_id
             FROM store_operations WHERE operation_id = ?1",
            [operation_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::database)?;
    row.map(|(request_digest, state, result_object_id)| {
        OperationRecord::new(
            operation_id.clone(),
            RequestDigest::new(request_digest)?,
            OperationState::parse(&state)?,
            result_object_id.map(ObjectId::new).transpose()?,
        )
    })
    .transpose()
}

fn read_lease(connection: &Connection, lease_id: &LeaseId) -> StoreResult<Option<LeaseRecord>> {
    let row = connection
        .query_row(
            "SELECT object_id, holder, expires_after FROM store_leases WHERE lease_id = ?1",
            [lease_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::database)?;
    row.map(|(object_id, holder, expires_after)| {
        LeaseRecord::new(
            lease_id.clone(),
            ObjectId::new(object_id)?,
            holder.into(),
            LogicalEpoch::new(u64::try_from(expires_after).map_err(|_| {
                StoreError::new(
                    StoreErrorCode::IntegrityViolation,
                    "stored lease epoch is invalid",
                )
            })?),
        )
    })
    .transpose()
}

fn epoch_to_i64(epoch: LogicalEpoch) -> StoreResult<i64> {
    i64::try_from(epoch.get()).map_err(|_| {
        StoreError::new(
            StoreErrorCode::LeaseInvalid,
            "logical lease epoch exceeds SQLite integer range",
        )
    })
}

fn table_count(connection: &Connection, table: &str) -> StoreResult<u64> {
    let sql = match table {
        "store_catalog" => "SELECT COUNT(*) FROM store_catalog",
        "store_operations" => "SELECT COUNT(*) FROM store_operations",
        "store_leases" => "SELECT COUNT(*) FROM store_leases",
        _ => {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "unknown internal table",
            ));
        }
    };
    let count: i64 = connection
        .query_row(sql, [], |row| row.get(0))
        .map_err(StoreError::database)?;
    u64::try_from(count).map_err(|_| {
        StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "stored record count is invalid",
        )
    })
}

fn query_ids(connection: &Connection, sql: &str, limit: u32) -> StoreResult<Vec<ObjectId>> {
    let mut statement = connection.prepare(sql).map_err(StoreError::database)?;
    let values = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(StoreError::database)?
        .map(|row| row.map_err(StoreError::database).and_then(ObjectId::new))
        .collect::<StoreResult<Vec<_>>>()?;
    if values.len() > limit as usize {
        return Err(StoreError::new(
            StoreErrorCode::BudgetExceeded,
            "logical manifest object budget exceeded",
        ));
    }
    Ok(values)
}

fn query_catalog(connection: &Connection, limit: u32) -> StoreResult<Vec<CatalogEntry>> {
    let mut statement = connection
        .prepare("SELECT catalog, path, object_id FROM store_catalog ORDER BY catalog, path")
        .map_err(StoreError::database)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(StoreError::database)?;
    let mut output = Vec::new();
    for row in rows {
        if output.len() == limit as usize {
            return Err(StoreError::new(
                StoreErrorCode::BudgetExceeded,
                "logical manifest catalog budget exceeded",
            ));
        }
        let (catalog, path, object_id) = row.map_err(StoreError::database)?;
        output.push(CatalogEntry::new(
            CatalogName::new(catalog)?,
            CatalogPath::new(path)?,
            ObjectId::new(object_id)?,
        ));
    }
    Ok(output)
}

fn query_operations(connection: &Connection, limit: u32) -> StoreResult<Vec<OperationRecord>> {
    let mut statement = connection
        .prepare(
            "SELECT operation_id, request_digest, state, result_object_id
             FROM store_operations ORDER BY operation_id",
        )
        .map_err(StoreError::database)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(StoreError::database)?;
    let mut output = Vec::new();
    for row in rows {
        if output.len() == limit as usize {
            return Err(StoreError::new(
                StoreErrorCode::BudgetExceeded,
                "logical manifest operation budget exceeded",
            ));
        }
        let (operation_id, request_digest, state, result_object_id) =
            row.map_err(StoreError::database)?;
        output.push(OperationRecord::new(
            OperationId::new(operation_id)?,
            RequestDigest::new(request_digest)?,
            OperationState::parse(&state)?,
            result_object_id.map(ObjectId::new).transpose()?,
        )?);
    }
    Ok(output)
}

fn query_leases(connection: &Connection, limit: u32) -> StoreResult<Vec<LeaseRecord>> {
    let mut statement = connection
        .prepare(
            "SELECT lease_id, object_id, holder, expires_after
             FROM store_leases ORDER BY lease_id",
        )
        .map_err(StoreError::database)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(StoreError::database)?;
    let mut output = Vec::new();
    for row in rows {
        if output.len() == limit as usize {
            return Err(StoreError::new(
                StoreErrorCode::BudgetExceeded,
                "logical manifest lease budget exceeded",
            ));
        }
        let (lease_id, object_id, holder, expires_after) = row.map_err(StoreError::database)?;
        output.push(LeaseRecord::new(
            LeaseId::new(lease_id)?,
            ObjectId::new(object_id)?,
            holder.into(),
            LogicalEpoch::new(u64::try_from(expires_after).map_err(|_| {
                StoreError::new(
                    StoreErrorCode::IntegrityViolation,
                    "stored lease epoch is invalid",
                )
            })?),
        )?);
    }
    Ok(output)
}
