mod bootstrap;
use std::{fs, io::Cursor, path::Path, time::Duration};

use rusqlite::{
    Connection, MAIN_DB, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
    types::ValueRef,
};

use crate::{
    CatalogChange, CatalogEntry, CatalogExpectation, CatalogMutation, CatalogName, CatalogPath,
    CommitReceipt, GarbageCollectionReceipt, IntegrityReport, LeaseId, LeaseRecord, LogicalEpoch,
    LogicalManifest, ObjectId, ObjectRecord, OperationBegin, OperationId, OperationRecord,
    OperationState, RequestDigest, StoreConfiguration, StoreError, StoreErrorCode, StoreResult,
    WriteBatch,
};

const APPLICATION_ID: i64 = 0x5744_4631;
const USER_VERSION: i64 = 1;

/// Busy window for the read-only admission probe of a candidate file.
const ADMISSION_TIMEOUT_MS: u64 = 500;

/// Maximum accepted physical schema object count, bounding the comparison.
const MAX_SCHEMA_OBJECTS: usize = 32;

/// Maximum length of an owned schema object name, bounded before any query.
const MAX_SCHEMA_NAME_BYTES: usize = 256;

/// Maximum length of an owned schema object definition, bounded before any query.
const MAX_SCHEMA_SQL_BYTES: usize = 8192;

/// Exact SQLite internal object names this compiled schema may contain.
const INTERNAL_SCHEMA_NAMES: &[&str] = &[
    "sqlite_autoindex_store_meta_1",
    "sqlite_autoindex_store_objects_1",
    "sqlite_autoindex_store_catalog_1",
    "sqlite_autoindex_store_operations_1",
    "sqlite_autoindex_store_leases_1",
];

/// Bounded physical-schema inspection. It describes objects, never runs them.
const SCHEMA_INSPECTION_SQL: &str = "SELECT type, name, tbl_name, sql FROM sqlite_schema LIMIT 33";

/// One owned description of this compiled store's physical schema.
///
/// The expected shape is derived from the crate's own DDL in a private in-memory
/// database, never from the file under admission. The comparison covers the exact
/// object kind, name and definition text, so altered columns, constraints, indexes,
/// or substituted views are a refusal rather than an adoption.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedSchemaShape {
    objects: Vec<OwnedSchemaObject>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OwnedSchemaObject {
    kind: String,
    name: String,
    table_name: String,
    sql: String,
}

impl OwnedSchemaShape {
    /// Builds the expected shape from the crate's own compiled DDL.
    fn compiled() -> StoreResult<Self> {
        let connection = Connection::open_in_memory().map_err(StoreError::database)?;
        initialize_schema(&connection)?;
        Self::observed(&connection)
    }

    /// Reads the exact physical shape from one already-configured connection.
    ///
    /// Every mandatory field is bound and length-checked as a borrowed `ValueRef`
    /// before any owned copy, and the row count is bounded, so a foreign schema
    /// cannot force an allocation or a silently skipped row.
    fn observed(connection: &Connection) -> StoreResult<Self> {
        let mut statement = connection
            .prepare(SCHEMA_INSPECTION_SQL)
            .map_err(StoreError::database)?;
        let mut rows = statement.query([]).map_err(StoreError::database)?;
        let mut objects = Vec::new();
        while let Some(row) = rows.next().map_err(StoreError::database)? {
            if objects.len() >= MAX_SCHEMA_OBJECTS {
                return Err(StoreError::new(
                    StoreErrorCode::IntegrityViolation,
                    "physical schema exceeds the reviewed object budget",
                ));
            }
            objects.push(Self::bounded_row(row)?);
        }
        objects.sort();
        Ok(Self { objects })
    }

    fn bounded_row(row: &rusqlite::Row<'_>) -> StoreResult<OwnedSchemaObject> {
        let kind = Self::bounded_owned_text(row.get_ref(0), 16, "schema object kind")?;
        let name =
            Self::bounded_owned_text(row.get_ref(1), MAX_SCHEMA_NAME_BYTES, "schema object name")?;
        let table_name =
            Self::bounded_owned_text(row.get_ref(2), MAX_SCHEMA_NAME_BYTES, "schema table name")?;
        let sql = match row.get_ref(3).map_err(StoreError::database)? {
            ValueRef::Null if kind == "index" && INTERNAL_SCHEMA_NAMES.contains(&name.as_str()) => {
                String::new()
            }
            _ => {
                Self::bounded_owned_text(row.get_ref(3), MAX_SCHEMA_SQL_BYTES, "schema definition")?
            }
        };
        Ok(OwnedSchemaObject {
            kind,
            name,
            table_name,
            sql,
        })
    }

    /// Binds one mandatory schema field, refusing a NULL and bounding its length before
    /// any owned allocation.
    fn bounded_owned_text(
        value: rusqlite::Result<ValueRef<'_>>,
        max_bytes: usize,
        label: &'static str,
    ) -> StoreResult<String> {
        match value {
            Ok(ValueRef::Text(bytes)) if bytes.len() > max_bytes => Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                format!("{label} exceeds the reviewed budget"),
            )),
            Ok(ValueRef::Text(bytes)) => std::str::from_utf8(bytes)
                .map(|text| text.to_owned())
                .map_err(|_| {
                    StoreError::new(
                        StoreErrorCode::IntegrityViolation,
                        format!("{label} is not valid UTF-8"),
                    )
                }),
            _ => Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                format!("{label} is missing or is not text"),
            )),
        }
    }
    fn matches_compiled(&self, compiled: &Self) -> bool {
        self.objects == compiled.objects
    }
}

/// Admission decision for one candidate database.
#[derive(Debug, Clone, PartialEq, Eq)]
enum AdmissionOutcome {
    /// The database carries no object owned by this compiled store.
    Fresh,
    /// The exact compiled schema, headers and configuration identity all match.
    Owned,
    /// The compiled schema and headers match but the configuration identity does
    /// not. This is a typed configuration refusal, distinct from a foreign file.
    ConfigMismatch,
    /// The database is foreign, incompatible, or holds a mismatched identity.
    Foreign,
}

/// Compares one candidate against the compiled schema before any named query.
///
/// Only an exact schema match proceeds to the configuration identity query, so a
/// substituted view is refused before it can run in place of an owned object. The
/// inspection enumerates every schema row, so an unregistered object is refused
/// rather than hidden by a name filter.
fn inspect_candidate(
    connection: &Connection,
    configuration: &StoreConfiguration,
    compiled: &OwnedSchemaShape,
    allow_initialize: bool,
) -> StoreResult<AdmissionOutcome> {
    let observed = OwnedSchemaShape::observed(connection)?;
    if !observed.matches_compiled(compiled) {
        // An empty database carries no object that could execute during a query.
        return Ok(if allow_initialize && observed.objects.is_empty() {
            AdmissionOutcome::Fresh
        } else {
            AdmissionOutcome::Foreign
        });
    }
    let application_id: i64 = connection
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .map_err(StoreError::database)?;
    let user_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(StoreError::database)?;
    if application_id != APPLICATION_ID || user_version != USER_VERSION {
        return Ok(AdmissionOutcome::Foreign);
    }
    let observed_configuration: Option<Option<String>> = connection
        .query_row(
            "SELECT CASE WHEN length(value)<=256 THEN value END FROM store_meta WHERE key='configuration_id'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::database)?;
    Ok(match observed_configuration.flatten() {
        Some(value) if value == configuration.configuration_id() => AdmissionOutcome::Owned,
        Some(_) => AdmissionOutcome::ConfigMismatch,
        None => AdmissionOutcome::Foreign,
    })
}

fn require_owned(outcome: AdmissionOutcome) -> StoreResult<()> {
    match outcome {
        AdmissionOutcome::Owned => Ok(()),
        AdmissionOutcome::ConfigMismatch => Err(StoreError::new(
            StoreErrorCode::ConfigurationInvalid,
            "store configuration does not match the durable database",
        )),
        AdmissionOutcome::Fresh | AdmissionOutcome::Foreign => Err(StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "store file does not match the compiled store layout and identity",
        )),
    }
}

/// One synchronous durable store. SQLite handles and physical schema stay private.
pub struct Store {
    connection: Connection,
    configuration: StoreConfiguration,
}

impl Store {
    /// Opens one owned store file, publishing a completely initialized file when
    /// it does not exist. Interrupted staging never occupies the final pathname.
    ///
    /// Creation uses an exclusive private staging directory and file, then an
    /// atomic no-replace hard-link publication on the same volume. On Unix the
    /// directory/file modes are 0700/0600. If the path already exists it
    /// is admitted read-only first: its physical schema, header identity and
    /// configuration identity must match this compiled store exactly before a
    /// writable handle opens or any DDL runs. A foreign, incompatible, or pre-existing
    /// empty file is refused with its bytes untouched. No caller-asserted ownership
    /// flag exists. This is a process-interruption boundary, not a claim of
    /// power-loss durability for the parent directory.
    pub fn open(path: impl AsRef<Path>, configuration: StoreConfiguration) -> StoreResult<Self> {
        configuration.limits().validate()?;
        let path = path.as_ref();
        match fs::symlink_metadata(path) {
            Ok(_) => admit_file(path, configuration),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                bootstrap::create(path, configuration)
            }
            Err(_) => Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "store path could not be inspected",
            )),
        }
    }

    pub fn open_in_memory(configuration: StoreConfiguration) -> StoreResult<Self> {
        let connection = Connection::open_in_memory().map_err(StoreError::database)?;
        Self::initialize_connection(connection, configuration)
    }

    /// Initializes a brand-new, store-created database.
    ///
    /// Only reached from the private staging and in-memory creator routes, which
    /// own the database by construction. A fresh empty database is initialized here.
    fn initialize_connection(
        connection: Connection,
        configuration: StoreConfiguration,
    ) -> StoreResult<Self> {
        configuration.limits().validate()?;
        configure_connection(&connection, false)?;
        let compiled = OwnedSchemaShape::compiled()?;
        // Fresh authorization is not public: only a database this call created empty
        // may proceed to initialization.
        if !matches!(
            inspect_candidate(&connection, &configuration, &compiled, true)?,
            AdmissionOutcome::Fresh
        ) {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "store database is not a fresh owned database",
            ));
        }
        initialize_schema(&connection)?;
        #[cfg(test)]
        bootstrap::interrupt_at("schema");
        validate_connection_identity(&connection, &configuration, true)?;
        Ok(Self {
            connection,
            configuration,
        })
    }

    /// Re-admits an already read-only-admitted existing file as writable.
    ///
    /// The physical schema, headers and configuration identity are re-checked on
    /// this writable handle with initialization forbidden, so a path swapped or
    /// removed between the read-only proof and this open can only ever be refused,
    /// never adopted or initialized. The RW handle is opened without CREATE.
    fn adopt_connection(
        connection: Connection,
        configuration: StoreConfiguration,
    ) -> StoreResult<Self> {
        configuration.limits().validate()?;
        configure_connection(&connection, false)?;
        let compiled = OwnedSchemaShape::compiled()?;
        require_owned(inspect_candidate(
            &connection,
            &configuration,
            &compiled,
            false,
        )?)?;
        validate_connection_identity(&connection, &configuration, false)?;
        Ok(Self {
            connection,
            configuration,
        })
    }
    /// Serializes one complete, integrity-checked SQLite database image.
    ///
    /// The returned bytes are detached from this mutable handle. Callers must drop
    /// or otherwise stop using the writer before treating the image as sealed.
    pub fn serialize_database(&self, max_bytes: u64) -> StoreResult<Box<[u8]>> {
        validate_serialized_limit(max_bytes)?;
        let integrity =
            self.validate_integrity(self.configuration.limits().max_manifest_records)?;
        if !integrity.complete() {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "store integrity validation was truncated before serialization",
            ));
        }
        let serialized = self
            .connection
            .serialize(MAIN_DB)
            .map_err(StoreError::database)?;
        let bytes: &[u8] = &serialized;
        if bytes.is_empty() || bytes.len() as u64 > max_bytes {
            return Err(StoreError::new(
                StoreErrorCode::ObjectTooLarge,
                "serialized store exceeds the configured byte budget",
            ));
        }
        Ok(bytes.to_vec().into_boxed_slice())
    }

    #[must_use]
    pub fn configuration(&self) -> &StoreConfiguration {
        &self.configuration
    }

    pub fn object(&self, object_id: &ObjectId) -> StoreResult<Option<ObjectRecord>> {
        read_object(
            &self.connection,
            object_id,
            self.configuration.limits().max_object_bytes,
        )
    }

    pub fn catalog_entry(
        &self,
        catalog: &CatalogName,
        path: &CatalogPath,
    ) -> StoreResult<Option<CatalogEntry>> {
        read_catalog_entry(&self.connection, catalog, path)
    }

    /// Reads one exact durable operation record without mutating the journal.
    pub fn operation(&self, operation_id: &OperationId) -> StoreResult<Option<OperationRecord>> {
        read_operation(&self.connection, operation_id)
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
                let existing =
                    read_object(&transaction, object.object_id(), limits.max_object_bytes)?
                        .ok_or_else(|| {
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
            changes.push(apply_catalog_mutation(
                &transaction,
                mutation,
                limits.max_object_bytes,
            )?);
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
        let max_object_bytes = self.configuration.limits().max_object_bytes;
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
            && read_object(&transaction, object_id, max_object_bytes)?.is_none()
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
        let max_object_bytes = self.configuration.limits().max_object_bytes;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        if read_object(&transaction, &object_id, max_object_bytes)?.is_none() {
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
        validate_connection_integrity(&self.connection, &self.configuration, max_objects)
    }

    pub fn logical_manifest(&self) -> StoreResult<LogicalManifest> {
        build_logical_manifest(&self.connection, &self.configuration)
    }
}

/// Inspects one existing file read-only and admits it before any write.
///
/// The compiled schema is compared first. A foreign, incompatible, or
/// non-owned file never reaches a writable handle, so its bytes,
/// schema, and sidecars are preserved exactly. Only an exact owned match
/// opens a writable connection.
fn admit_file(path: &Path, configuration: StoreConfiguration) -> StoreResult<Store> {
    // Also check a winner created while our private bootstrap was running.
    let existing = fs::symlink_metadata(path).map_err(|_| {
        StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "store path could not be inspected",
        )
    })?;
    #[cfg(windows)]
    let reparse = {
        use std::os::windows::fs::MetadataExt;
        existing.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let reparse = false;
    if !existing.file_type().is_file() || existing.file_type().is_symlink() || reparse {
        return Err(StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "store path is not a regular file",
        ));
    }
    let compiled = OwnedSchemaShape::compiled()?;
    let inspect = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(StoreError::database)?;
    configure_connection(&inspect, true)?;
    inspect
        .busy_timeout(Duration::from_millis(ADMISSION_TIMEOUT_MS))
        .map_err(StoreError::database)?;
    let outcome = inspect_candidate(&inspect, &configuration, &compiled, false)?;
    drop(inspect);
    require_owned(outcome)?;
    // Reopen read-write without CREATE and re-admit as Owned only, on
    // the exact handle that will run the store.
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(StoreError::database)?;
    Store::adopt_connection(connection, configuration)
}
/// Read-only handle over a detached serialized store image.
///
/// The SQLite image is deserialized with SQLite's read-only flag and the public
/// type exposes no mutation methods. Configuration, schema and integrity are
/// revalidated independently during open.
pub struct SealedStore {
    connection: Connection,
    configuration: StoreConfiguration,
}

impl SealedStore {
    pub fn open_serialized(
        bytes: &[u8],
        configuration: StoreConfiguration,
        max_bytes: u64,
    ) -> StoreResult<Self> {
        configuration.limits().validate()?;
        validate_serialized_limit(max_bytes)?;
        if bytes.is_empty() || bytes.len() as u64 > max_bytes {
            return Err(StoreError::new(
                StoreErrorCode::ObjectTooLarge,
                "serialized store is empty or exceeds the configured byte budget",
            ));
        }
        let mut connection = Connection::open_in_memory().map_err(StoreError::database)?;
        connection
            .deserialize_read_exact(MAIN_DB, Cursor::new(bytes), bytes.len(), true)
            .map_err(StoreError::database)?;
        configure_connection(&connection, true)?;
        // Admit the exact compiled physical schema before any named object
        // query. A substituted view, altered constraint, or extra object is
        // refused here, before it can execute in place of an owned table.
        let compiled = OwnedSchemaShape::compiled()?;
        require_owned(inspect_candidate(
            &connection,
            &configuration,
            &compiled,
            false,
        )?)?;
        validate_connection_identity(&connection, &configuration, false)?;
        let sealed = Self {
            connection,
            configuration,
        };
        let integrity =
            sealed.validate_integrity(sealed.configuration.limits().max_manifest_records)?;
        if !integrity.complete() {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "sealed store integrity validation was truncated",
            ));
        }
        Ok(sealed)
    }

    #[must_use]
    pub fn configuration(&self) -> &StoreConfiguration {
        &self.configuration
    }

    pub fn object(&self, object_id: &ObjectId) -> StoreResult<Option<ObjectRecord>> {
        read_object(
            &self.connection,
            object_id,
            self.configuration.limits().max_object_bytes,
        )
    }

    pub fn catalog_entry(
        &self,
        catalog: &CatalogName,
        path: &CatalogPath,
    ) -> StoreResult<Option<CatalogEntry>> {
        read_catalog_entry(&self.connection, catalog, path)
    }

    /// Reads one exact durable operation record without mutating the journal.
    pub fn operation(&self, operation_id: &OperationId) -> StoreResult<Option<OperationRecord>> {
        read_operation(&self.connection, operation_id)
    }

    pub fn validate_integrity(&self, max_objects: u32) -> StoreResult<IntegrityReport> {
        validate_connection_integrity(&self.connection, &self.configuration, max_objects)
    }

    pub fn logical_manifest(&self) -> StoreResult<LogicalManifest> {
        build_logical_manifest(&self.connection, &self.configuration)
    }
}

fn configure_connection(connection: &Connection, read_only: bool) -> StoreResult<()> {
    use rusqlite::config::DbConfig;
    connection
        .set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)
        .map_err(StoreError::database)?;
    connection
        .set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER, false)
        .map_err(StoreError::database)?;
    connection
        .set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_VIEW, false)
        .map_err(StoreError::database)?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(StoreError::database)?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA trusted_schema = OFF;")
        .map_err(StoreError::database)?;
    if read_only {
        connection
            .pragma_update(None, "query_only", true)
            .map_err(StoreError::database)?;
    } else {
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(StoreError::database)?;
    }
    Ok(())
}

fn initialize_schema(connection: &Connection) -> StoreResult<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS store_meta (
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
    Ok(())
}

fn validate_connection_identity(
    connection: &Connection,
    configuration: &StoreConfiguration,
    initialize: bool,
) -> StoreResult<()> {
    let application_id: i64 = connection
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .map_err(StoreError::database)?;
    if application_id == 0 && initialize {
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
    if user_version == 0 && initialize {
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
        Some(value) if value != configuration.configuration_id() => Err(StoreError::new(
            StoreErrorCode::ConfigurationInvalid,
            "store configuration does not match the durable database",
        )),
        Some(_) => Ok(()),
        None if initialize => {
            connection
                .execute(
                    "INSERT INTO store_meta(key, value) VALUES ('configuration_id', ?1)",
                    [configuration.configuration_id()],
                )
                .map_err(StoreError::database)?;
            Ok(())
        }
        None => Err(StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "sealed store lacks its exact configuration identity",
        )),
    }
}

fn validate_serialized_limit(max_bytes: u64) -> StoreResult<()> {
    if max_bytes == 0 || max_bytes > 512 * 1024 * 1024 {
        return Err(StoreError::new(
            StoreErrorCode::ConfigurationInvalid,
            "serialized store byte limit is outside the reviewed profile",
        ));
    }
    Ok(())
}

fn read_catalog_entry(
    connection: &Connection,
    catalog: &CatalogName,
    path: &CatalogPath,
) -> StoreResult<Option<CatalogEntry>> {
    let object_id: Option<String> = connection
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

fn validate_connection_integrity(
    connection: &Connection,
    configuration: &StoreConfiguration,
    max_objects: u32,
) -> StoreResult<IntegrityReport> {
    if max_objects == 0 || max_objects > configuration.limits().max_manifest_records {
        return Err(StoreError::new(
            StoreErrorCode::BudgetExceeded,
            "integrity object budget is invalid",
        ));
    }
    let sqlite: String = connection
        .query_row("PRAGMA integrity_check(1)", [], |row| row.get(0))
        .map_err(StoreError::database)?;
    if sqlite != "ok" {
        return Err(StoreError::new(
            StoreErrorCode::IntegrityViolation,
            "SQLite integrity validation failed",
        ));
    }
    let mut statement = connection
        .prepare(
            "SELECT object_id, kind, schema_version, content_sha256,
                    CASE WHEN length(canonical_json) <= ?2 THEN canonical_json END
             FROM store_objects ORDER BY object_id LIMIT ?1",
        )
        .map_err(StoreError::database)?;
    let max_object_bytes =
        i64::try_from(configuration.limits().max_object_bytes).map_err(|_| {
            StoreError::new(
                StoreErrorCode::ConfigurationInvalid,
                "object byte limit is outside the reviewed profile",
            )
        })?;
    let rows = statement
        .query_map(
            params![i64::from(max_objects) + 1, max_object_bytes],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                ))
            },
        )
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
        // A persisted row larger than the selected ceiling is refused here,
        // before any complete report can be produced from it.
        let Some(canonical_json) = canonical_json else {
            return Err(StoreError::new(
                StoreErrorCode::ObjectTooLarge,
                "stored object exceeds the configured byte budget",
            ));
        };
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
    let catalog = table_count(connection, "store_catalog")?;
    let operations = table_count(connection, "store_operations")?;
    let leases = table_count(connection, "store_leases")?;
    let foreign_keys: i64 = connection
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

fn build_logical_manifest(
    connection: &Connection,
    configuration: &StoreConfiguration,
) -> StoreResult<LogicalManifest> {
    let limit = configuration.limits().max_manifest_records;
    let objects = query_ids(
        connection,
        "SELECT object_id FROM store_objects ORDER BY object_id",
        limit,
    )?;
    let catalog_entries = query_catalog(connection, limit)?;
    let operations = query_operations(connection, limit)?;
    let leases = query_leases(connection, limit)?;
    LogicalManifest::build(
        configuration.configuration_id().into(),
        objects,
        catalog_entries,
        operations,
        leases,
    )
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
    max_object_bytes: u64,
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
        if read_object(transaction, target, max_object_bytes)?.is_none() {
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

fn read_object(
    connection: &Connection,
    object_id: &ObjectId,
    max_object_bytes: u64,
) -> StoreResult<Option<ObjectRecord>> {
    // Admit the borrowed row length before any owned payload materialization,
    // on the same held statement and snapshot as the read itself. An oversized
    // row yields a NULL payload here and is refused before allocation.
    let limit = i64::try_from(max_object_bytes).map_err(|_| {
        StoreError::new(
            StoreErrorCode::ConfigurationInvalid,
            "object byte limit is outside the reviewed profile",
        )
    })?;
    let row = connection
        .query_row(
            "SELECT kind, schema_version, content_sha256,
                    CASE WHEN length(canonical_json) <= ?2 THEN canonical_json END
             FROM store_objects WHERE object_id = ?1",
            params![object_id.as_str(), limit],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::database)?;
    row.map(|(kind, schema_version, content_sha256, canonical_json)| {
        let Some(canonical_json) = canonical_json else {
            return Err(StoreError::new(
                StoreErrorCode::ObjectTooLarge,
                "stored object exceeds the configured byte budget",
            ));
        };
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
