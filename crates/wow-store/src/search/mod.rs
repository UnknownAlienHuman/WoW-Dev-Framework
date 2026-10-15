//! Immutable, content-addressed SQLite/FTS5 search shard storage.
//!
//! This module owns the physical SQLite image, schema admission, serialization,
//! read-only/query-only reopening, and the generic Store lease that retains it.
//! Logical projection and query authority remain in `wow-search`.

use std::{
    io::Cursor,
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
};

use rusqlite::{Connection, MAIN_DB, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    LeaseId, LeaseRecord, LogicalEpoch, ObjectId, PendingObject, Store, StoreError, StoreErrorCode,
    StoreResult, WriteBatch,
};

const PHYSICAL_SCHEMA: &str = "wow-store/search-shard-sqlite-fts5/1";
const OBJECT_KIND: &str = "wow.search.shard.sqlite-fts5";
const OBJECT_SCHEMA_VERSION: u32 = 1;
const MAX_SCHEMA_OBJECTS: usize = 64;
const SCHEMA_INSPECTION_SQL: &str = "SELECT type, name, tbl_name, sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name LIMIT 65";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchStoreLimits {
    max_documents: u32,
    max_document_id_bytes: u32,
    max_owner_metadata_label_bytes: u32,
    max_literal_query_bytes: u32,
    max_candidate_results: u32,
    max_owner_manifest_bytes: u64,
    max_database_bytes: u64,
    max_store_object_bytes: u64,
}

impl SearchStoreLimits {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_documents: u32,
        max_document_id_bytes: u32,
        max_owner_metadata_label_bytes: u32,
        max_literal_query_bytes: u32,
        max_candidate_results: u32,
        max_owner_manifest_bytes: u64,
        max_database_bytes: u64,
        max_store_object_bytes: u64,
    ) -> StoreResult<Self> {
        let value = Self {
            max_documents,
            max_document_id_bytes,
            max_owner_metadata_label_bytes,
            max_literal_query_bytes,
            max_candidate_results,
            max_owner_manifest_bytes,
            max_database_bytes,
            max_store_object_bytes,
        };
        if [
            u64::from(max_documents),
            u64::from(max_document_id_bytes),
            u64::from(max_owner_metadata_label_bytes),
            u64::from(max_literal_query_bytes),
            u64::from(max_candidate_results),
            max_owner_manifest_bytes,
            max_database_bytes,
            max_store_object_bytes,
        ]
        .contains(&0)
            || max_database_bytes > max_store_object_bytes
        {
            return Err(invalid_profile());
        }
        Ok(value)
    }

    #[must_use]
    pub const fn max_documents(self) -> u32 {
        self.max_documents
    }
}

/// A profile identity is supplied by the logical search owner and binds every
/// limit in this physical profile. This module validates shape, not policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStoreProfile {
    profile_digest: Box<str>,
    limits: SearchStoreLimits,
}

impl SearchStoreProfile {
    pub fn new(
        profile_digest: impl Into<Box<str>>,
        limits: SearchStoreLimits,
    ) -> StoreResult<Self> {
        let profile_digest = profile_digest.into();
        if !valid_digest(&profile_digest) {
            return Err(invalid_profile());
        }
        Ok(Self {
            profile_digest,
            limits,
        })
    }

    #[must_use]
    pub fn profile_digest(&self) -> &str {
        &self.profile_digest
    }

    #[must_use]
    pub const fn limits(&self) -> SearchStoreLimits {
        self.limits
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStoreDocument {
    document_id: Box<str>,
    owner_metadata_label: Box<str>,
}

impl SearchStoreDocument {
    #[must_use]
    pub fn new(
        document_id: impl Into<Box<str>>,
        owner_metadata_label: impl Into<Box<str>>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            owner_metadata_label: owner_metadata_label.into(),
        }
    }

    #[must_use]
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    #[must_use]
    pub fn owner_metadata_label(&self) -> &str {
        &self.owner_metadata_label
    }
}

impl Serialize for SearchStoreDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Fields<'a> {
            document_id: &'a str,
            owner_metadata_label: &'a str,
        }
        Fields {
            document_id: &self.document_id,
            owner_metadata_label: &self.owner_metadata_label,
        }
        .serialize(serializer)
    }
}

#[derive(Debug, Clone)]
pub struct SearchStoreLeaseRequest {
    lease_id: LeaseId,
    holder: Box<str>,
    expires_after: LogicalEpoch,
}

impl SearchStoreLeaseRequest {
    pub fn new(
        lease_id: LeaseId,
        holder: impl Into<Box<str>>,
        expires_after: LogicalEpoch,
    ) -> StoreResult<Self> {
        let holder = holder.into();
        if holder.is_empty()
            || holder.len() > 256
            || !holder
                .bytes()
                .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'"' | b'\\'))
        {
            return Err(StoreError::new(
                StoreErrorCode::LeaseInvalid,
                "invalid search shard lease holder",
            ));
        }
        Ok(Self {
            lease_id,
            holder,
            expires_after,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStoreCandidate {
    document_id: Box<str>,
    owner_metadata_label: Box<str>,
}

impl SearchStoreCandidate {
    #[must_use]
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    #[must_use]
    pub fn owner_metadata_label(&self) -> &str {
        &self.owner_metadata_label
    }
}

/// A published generic-store object with a real lease and a fresh read-only
/// SQLite view over its embedded immutable image.
pub struct PublishedSearchShard {
    object_id: ObjectId,
    search_shard_id: Box<str>,
    lease: LeaseRecord,
    reader: SearchStoreReader,
}

/// A validated immutable shard object before it is committed to Store.
/// Service can publish the object, exact catalog root, and response atomically.
pub struct PreparedSearchShard {
    object: PendingObject,
    search_shard_id: Box<str>,
}

impl PreparedSearchShard {
    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        self.object.object_id()
    }

    #[must_use]
    pub fn search_shard_id(&self) -> &str {
        &self.search_shard_id
    }

    #[must_use]
    pub fn into_object(self) -> PendingObject {
        self.object
    }
}

impl PublishedSearchShard {
    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn search_shard_id(&self) -> &str {
        &self.search_shard_id
    }

    #[must_use]
    pub fn lease(&self) -> &LeaseRecord {
        &self.lease
    }

    #[must_use]
    pub fn reader(&self) -> &SearchStoreReader {
        &self.reader
    }

    /// Release the exact lease only after dropping its read-only SQLite view.
    pub fn release(self, store: &mut Store) -> StoreResult<()> {
        let Self { lease, reader, .. } = self;
        drop(reader);
        store.release_lease(lease.lease_id(), lease.holder())
    }
}

pub struct SearchStoreReader {
    connection: Connection,
    profile: SearchStoreProfile,
    manifest_bytes: Box<[u8]>,
    document_count: u32,
}

impl SearchStoreReader {
    #[must_use]
    pub fn profile_digest(&self) -> &str {
        self.profile.profile_digest()
    }

    /// The logical owner manifest is opaque to the physical store.
    #[must_use]
    pub fn owner_manifest_bytes(&self) -> &[u8] {
        &self.manifest_bytes
    }

    #[must_use]
    pub const fn document_count(&self) -> u32 {
        self.document_count
    }

    pub fn exact_id(&self, document_id: &str) -> StoreResult<Option<SearchStoreDocument>> {
        if document_id.len() > self.profile.limits.max_document_id_bytes as usize {
            return Err(budget_refused());
        }
        let value = self
            .connection
            .query_row(
                "SELECT document_id, owner_metadata_label FROM search_documents WHERE document_id = ?1 COLLATE BINARY",
                [document_id],
                |row| {
                    Ok(SearchStoreDocument::new(
                        row.get::<_, String>(0)?.into_boxed_str(),
                        row.get::<_, String>(1)?.into_boxed_str(),
                    ))
                },
            )
            .optional()
            .map_err(StoreError::database)?;
        Ok(value)
    }

    /// Exact byte-sensitive label relation. Ambiguity above the supplied cap
    /// refuses the entire result; it never returns a truncated exact set.
    pub fn exact_label(&self, label: &str) -> StoreResult<Vec<SearchStoreDocument>> {
        if label.len() > self.profile.limits.max_owner_metadata_label_bytes as usize {
            return Err(budget_refused());
        }
        let cap = self.profile.limits.max_candidate_results;
        let query_limit = i64::from(cap) + 1;
        let mut statement = self.connection.prepare(
            "SELECT document_id, owner_metadata_label FROM search_documents WHERE owner_metadata_label = ?1 COLLATE BINARY LIMIT ?2",
        ).map_err(StoreError::database)?;
        let rows = statement
            .query_map(params![label, query_limit], |row| {
                Ok(SearchStoreDocument::new(
                    row.get::<_, String>(0)?.into_boxed_str(),
                    row.get::<_, String>(1)?.into_boxed_str(),
                ))
            })
            .map_err(StoreError::database)?;
        let mut result = Vec::new();
        for row in rows {
            if result.len() >= cap as usize {
                return Err(budget_refused());
            }
            result.push(row.map_err(StoreError::database)?);
        }
        Ok(result)
    }

    /// Closed literal phrase query over the one allowlisted metadata label.
    /// The term is quoted and bound internally; callers cannot pass MATCH syntax.
    pub fn literal_candidates(&self, literal: &str) -> StoreResult<Vec<SearchStoreCandidate>> {
        validate_literal(literal, self.profile.limits.max_literal_query_bytes)?;
        let match_expression = quote_fts_literal(literal);
        let cap = self.profile.limits.max_candidate_results;
        let query_limit = i64::from(cap) + 1;
        let mut statement = self.connection.prepare(
            "SELECT d.ordinal, d.document_id, d.owner_metadata_label FROM search_fts JOIN search_documents AS d ON d.ordinal = search_fts.rowid WHERE search_fts MATCH ?1 LIMIT ?2",
        ).map_err(StoreError::database)?;
        let rows = statement
            .query_map(params![match_expression, query_limit], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(StoreError::database)?;
        let mut found = Vec::new();
        for row in rows {
            if found.len() >= cap as usize {
                return Err(budget_refused());
            }
            found.push(row.map_err(StoreError::database)?);
        }
        found.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.as_bytes().cmp(right.1.as_bytes()))
        });
        Ok(found
            .into_iter()
            .map(
                |(_, document_id, owner_metadata_label)| SearchStoreCandidate {
                    document_id: document_id.into_boxed_str(),
                    owner_metadata_label: owner_metadata_label.into_boxed_str(),
                },
            )
            .collect())
    }
}

pub struct SearchStore;

impl SearchStore {
    /// Builds, validates, and seals an immutable physical shard without Store
    /// writes. Its caller publishes the object in the owning operation batch.
    pub fn prepare(
        profile: SearchStoreProfile,
        search_shard_id: impl Into<Box<str>>,
        documents: &[SearchStoreDocument],
        owner_manifest_bytes: &[u8],
        object_limits: crate::StoreLimits,
        stop: &AtomicBool,
    ) -> StoreResult<PreparedSearchShard> {
        check_cancelled(stop)?;
        let search_shard_id = search_shard_id.into();
        validate_search_shard_id(&search_shard_id)?;
        if documents.is_empty()
            || documents.len() > profile.limits.max_documents as usize
            || owner_manifest_bytes.is_empty()
            || owner_manifest_bytes.len() as u64 > profile.limits.max_owner_manifest_bytes
            || profile.limits.max_store_object_bytes > object_limits.max_object_bytes
        {
            return Err(budget_refused());
        }
        let documents = canonical_documents(documents, profile.limits, stop)?;
        let manifest_sha256 = format!("sha256:{}", hex(&Sha256::digest(owner_manifest_bytes)));
        let database = build_database(
            &profile,
            &search_shard_id,
            &manifest_sha256,
            &documents,
            stop,
        )?;
        if database.len() as u64 > profile.limits.max_database_bytes {
            return Err(budget_refused());
        }
        let stored = StoredSearchArtifact {
            schema: PHYSICAL_SCHEMA.to_owned(),
            search_shard_id: search_shard_id.to_string(),
            profile: SearchStoreProfileWire {
                profile_digest: profile.profile_digest().to_owned(),
                limits: profile.limits,
            },
            document_count: u32::try_from(documents.len()).map_err(|_| budget_refused())?,
            manifest_sha256,
            owner_manifest_hex: hex(owner_manifest_bytes),
            database_sha256: format!("sha256:{}", hex(&Sha256::digest(&database))),
            database_hex: hex(&database),
        };
        let object =
            PendingObject::from_json(OBJECT_KIND, OBJECT_SCHEMA_VERSION, &stored, object_limits)?;
        if object.canonical_json().len() as u64 > profile.limits.max_store_object_bytes {
            return Err(budget_refused());
        }
        let reader = open_read_only(&stored, &profile, &search_shard_id, stop)?;
        if reader.owner_manifest_bytes() != owner_manifest_bytes
            || reader.document_count() != documents.len() as u32
        {
            return Err(integrity_failure());
        }
        drop(reader);
        Ok(PreparedSearchShard {
            object,
            search_shard_id,
        })
    }

    /// Opens one caller-selected object and shard ID under a fresh Store lease.
    /// The caller must separately validate its exact catalog/root membership.
    pub fn open_exact(
        store: &mut Store,
        profile: SearchStoreProfile,
        search_shard_id: impl Into<Box<str>>,
        object_id: &ObjectId,
        lease_request: SearchStoreLeaseRequest,
        stop: &AtomicBool,
    ) -> StoreResult<PublishedSearchShard> {
        check_cancelled(stop)?;
        let search_shard_id = search_shard_id.into();
        validate_search_shard_id(&search_shard_id)?;
        if profile.limits.max_store_object_bytes > store.configuration().limits().max_object_bytes {
            return Err(budget_refused());
        }
        let lease = store.acquire_lease(
            lease_request.lease_id,
            object_id.clone(),
            lease_request.holder,
            lease_request.expires_after,
        )?;
        let expected_shard_id = search_shard_id.clone();
        finish_after_lease(store, lease, search_shard_id, move |store, lease| {
            check_cancelled(stop)?;
            let record = store.object(lease.object_id())?.ok_or_else(|| {
                StoreError::new(
                    StoreErrorCode::ObjectMissing,
                    "published search shard object is missing",
                )
            })?;
            if record.object_id() != lease.object_id()
                || record.kind() != OBJECT_KIND
                || record.schema_version() != OBJECT_SCHEMA_VERSION
                || record.canonical_json().len() as u64 > profile.limits.max_store_object_bytes
            {
                return Err(integrity_failure());
            }
            let persisted: StoredSearchArtifact = record.decode()?;
            let reader = open_read_only(&persisted, &profile, &expected_shard_id, stop)?;
            check_cancelled(stop)?;
            Ok(reader)
        })
    }

    /// Build into a fresh private SQLite image, validate it, persist the complete
    /// image and owner manifest as one generic content-addressed object, lease it,
    /// then reopen that exact object read-only/query-only before returning.
    #[allow(clippy::too_many_arguments)]
    pub fn build_and_publish(
        store: &mut Store,
        profile: SearchStoreProfile,
        search_shard_id: impl Into<Box<str>>,
        documents: &[SearchStoreDocument],
        owner_manifest_bytes: &[u8],
        lease_request: SearchStoreLeaseRequest,
        stop: &AtomicBool,
    ) -> StoreResult<PublishedSearchShard> {
        let prepared = Self::prepare(
            profile.clone(),
            search_shard_id,
            documents,
            owner_manifest_bytes,
            store.configuration().limits(),
            stop,
        )?;
        let object_id = prepared.object_id().clone();
        let search_shard_id = prepared.search_shard_id().to_owned().into_boxed_str();
        let mut batch = WriteBatch::new();
        batch.add_object(prepared.into_object())?;
        store.commit(batch)?;
        Self::open_exact(
            store,
            profile,
            search_shard_id,
            &object_id,
            lease_request,
            stop,
        )
    }
}

fn validate_search_shard_id(search_shard_id: &str) -> StoreResult<()> {
    if search_shard_id.is_empty()
        || search_shard_id.len() > 256
        || !search_shard_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".:_-".contains(&byte))
    {
        Err(invalid_profile())
    } else {
        Ok(())
    }
}

/// Completes the read-only reopen after lease acquisition, releasing the exact
/// lease before returning any failure from the post-acquisition path.
fn finish_after_lease<F>(
    store: &mut Store,
    lease: LeaseRecord,
    search_shard_id: Box<str>,
    open_reader: F,
) -> StoreResult<PublishedSearchShard>
where
    F: FnOnce(&Store, &LeaseRecord) -> StoreResult<SearchStoreReader>,
{
    match open_reader(store, &lease) {
        Ok(reader) => Ok(PublishedSearchShard {
            object_id: lease.object_id().clone(),
            search_shard_id,
            lease,
            reader,
        }),
        Err(operation_error) => match store.release_lease(lease.lease_id(), lease.holder()) {
            Ok(()) => Err(operation_error),
            Err(cleanup_error) => Err(StoreError::new(
                StoreErrorCode::OutcomeUnknown,
                format!(
                    "search shard post-lease operation failed ({:?}); release failed ({:?}); reconcile lease {} held by {} for object {} before retry (expires_after={})",
                    operation_error.code(),
                    cleanup_error.code(),
                    lease.lease_id().as_str(),
                    lease.holder(),
                    lease.object_id().as_str(),
                    lease.expires_after().get(),
                ),
            )),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredSearchArtifact {
    schema: String,
    search_shard_id: String,
    profile: SearchStoreProfileWire,
    document_count: u32,
    manifest_sha256: String,
    owner_manifest_hex: String,
    database_sha256: String,
    database_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchStoreProfileWire {
    profile_digest: String,
    limits: SearchStoreLimits,
}

impl Serialize for SearchStoreProfile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SearchStoreProfileWire {
            profile_digest: self.profile_digest.to_string(),
            limits: self.limits,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SearchStoreProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = SearchStoreProfileWire::deserialize(deserializer)?;
        if !valid_digest(&wire.profile_digest) {
            return Err(serde::de::Error::custom("invalid search profile digest"));
        }
        Ok(Self {
            profile_digest: wire.profile_digest.into_boxed_str(),
            limits: wire.limits,
        })
    }
}

fn canonical_documents(
    documents: &[SearchStoreDocument],
    limits: SearchStoreLimits,
    stop: &AtomicBool,
) -> StoreResult<Vec<SearchStoreDocument>> {
    let mut ordered = documents.to_vec();
    for document in &ordered {
        check_cancelled(stop)?;
        if document.document_id.is_empty()
            || document.owner_metadata_label.is_empty()
            || document.document_id.len() > limits.max_document_id_bytes as usize
            || document.owner_metadata_label.len() > limits.max_owner_metadata_label_bytes as usize
            || contains_control(&document.document_id)
            || contains_control(&document.owner_metadata_label)
        {
            return Err(budget_refused());
        }
    }
    ordered.sort_by(|left, right| {
        left.document_id
            .as_bytes()
            .cmp(right.document_id.as_bytes())
    });
    if ordered
        .windows(2)
        .any(|pair| pair[0].document_id == pair[1].document_id)
    {
        return Err(integrity_failure());
    }
    Ok(ordered)
}

#[derive(Serialize)]
struct SearchDocumentManifest<'a> {
    schema: &'static str,
    profile_digest: &'a str,
    documents: &'a [SearchStoreDocument],
}

fn build_database(
    profile: &SearchStoreProfile,
    search_shard_id: &str,
    manifest_sha256: &str,
    documents: &[SearchStoreDocument],
    stop: &AtomicBool,
) -> StoreResult<Box<[u8]>> {
    let mut connection = Connection::open_in_memory().map_err(StoreError::database)?;
    configure_connection(&connection, false)?;
    create_schema(&connection)?;
    let transaction = connection.transaction().map_err(StoreError::database)?;
    for (index, document) in documents.iter().enumerate() {
        check_cancelled(stop)?;
        let ordinal = i64::try_from(index + 1).map_err(|_| budget_refused())?;
        transaction
            .execute(
                "INSERT INTO search_documents(ordinal, document_id, owner_metadata_label) VALUES (?1, ?2, ?3)",
                params![ordinal, document.document_id(), document.owner_metadata_label()],
            )
            .map_err(StoreError::database)?;
        transaction
            .execute(
                "INSERT INTO search_fts(rowid, document_id, owner_metadata_label) VALUES (?1, ?2, ?3)",
                params![ordinal, document.document_id(), document.owner_metadata_label()],
            )
            .map_err(StoreError::database)?;
    }
    let documents_manifest = wow_core::canonical_json_bytes(&SearchDocumentManifest {
        schema: PHYSICAL_SCHEMA,
        profile_digest: profile.profile_digest(),
        documents,
    })
    .map_err(|_| invalid_profile())?;
    let documents_manifest_sha256 = format!("sha256:{}", hex(&Sha256::digest(documents_manifest)));
    let document_count = i64::try_from(documents.len()).map_err(|_| budget_refused())?;
    transaction
        .execute(
            "INSERT INTO search_meta(singleton, schema_version, search_shard_id, profile_digest, owner_manifest_sha256, documents_manifest_sha256, document_count) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
            params![PHYSICAL_SCHEMA, search_shard_id, profile.profile_digest(), manifest_sha256, documents_manifest_sha256, document_count],
        )
        .map_err(StoreError::database)?;
    transaction.commit().map_err(StoreError::database)?;
    validate_schema(&connection)?;
    validate_database(
        &connection,
        profile,
        search_shard_id,
        manifest_sha256,
        documents,
        stop,
    )?;
    connection
        .execute(
            "INSERT INTO search_fts(search_fts) VALUES ('integrity-check')",
            [],
        )
        .map_err(StoreError::database)?;
    check_sqlite_integrity(&connection)?;
    let serialized = connection
        .serialize(MAIN_DB)
        .map_err(StoreError::database)?;
    let bytes: &[u8] = &serialized;
    if bytes.is_empty() || bytes.len() as u64 > profile.limits.max_database_bytes {
        return Err(budget_refused());
    }
    Ok(bytes.to_vec().into_boxed_slice())
}

fn open_read_only(
    stored: &StoredSearchArtifact,
    expected_profile: &SearchStoreProfile,
    expected_shard_id: &str,
    stop: &AtomicBool,
) -> StoreResult<SearchStoreReader> {
    check_cancelled(stop)?;
    if stored.schema != PHYSICAL_SCHEMA
        || stored.search_shard_id != expected_shard_id
        || stored.profile.profile_digest != expected_profile.profile_digest()
        || stored.profile.limits != expected_profile.limits
        || !valid_digest(&stored.manifest_sha256)
        || !valid_digest(&stored.database_sha256)
        || stored.document_count == 0
        || stored.document_count > expected_profile.limits.max_documents
    {
        return Err(integrity_failure());
    }
    let manifest_bytes = decode_hex_bounded(
        &stored.owner_manifest_hex,
        expected_profile.limits.max_owner_manifest_bytes,
    )?;
    if format!("sha256:{}", hex(&Sha256::digest(&manifest_bytes))) != stored.manifest_sha256 {
        return Err(integrity_failure());
    }
    let database = decode_hex_bounded(
        &stored.database_hex,
        expected_profile.limits.max_database_bytes,
    )?;
    if format!("sha256:{}", hex(&Sha256::digest(&database))) != stored.database_sha256 {
        return Err(integrity_failure());
    }

    let validation = deserialize_database(&database, false)?;
    configure_connection(&validation, false)?;
    validate_schema(&validation)?;
    let documents = read_and_validate_documents(&validation, expected_profile.limits)?;
    if documents.len() != stored.document_count as usize {
        return Err(integrity_failure());
    }
    validate_database(
        &validation,
        expected_profile,
        expected_shard_id,
        &stored.manifest_sha256,
        &documents,
        stop,
    )?;
    validation
        .execute(
            "INSERT INTO search_fts(search_fts) VALUES ('integrity-check')",
            [],
        )
        .map_err(StoreError::database)?;
    check_sqlite_integrity(&validation)?;
    drop(validation);

    check_cancelled(stop)?;
    let connection = deserialize_database(&database, true)?;
    configure_connection(&connection, true)?;
    validate_schema(&connection)?;
    validate_database(
        &connection,
        expected_profile,
        expected_shard_id,
        &stored.manifest_sha256,
        &documents,
        stop,
    )?;
    check_sqlite_integrity(&connection)?;
    let query_only: i64 = connection
        .query_row("PRAGMA query_only", [], |row| row.get(0))
        .map_err(StoreError::database)?;
    if query_only != 1 {
        return Err(integrity_failure());
    }
    Ok(SearchStoreReader {
        connection,
        profile: expected_profile.clone(),
        manifest_bytes: manifest_bytes.into_boxed_slice(),
        document_count: u32::try_from(documents.len()).map_err(|_| budget_refused())?,
    })
}

fn deserialize_database(bytes: &[u8], read_only: bool) -> StoreResult<Connection> {
    if bytes.is_empty() {
        return Err(integrity_failure());
    }
    let mut connection = Connection::open_in_memory().map_err(StoreError::database)?;
    connection
        .deserialize_read_exact(MAIN_DB, Cursor::new(bytes), bytes.len(), read_only)
        .map_err(StoreError::database)?;
    Ok(connection)
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
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA trusted_schema = OFF;")
        .map_err(StoreError::database)?;
    if read_only {
        connection
            .pragma_update(None, "query_only", true)
            .map_err(StoreError::database)?;
    }
    Ok(())
}

fn create_schema(connection: &Connection) -> StoreResult<()> {
    connection
        .execute_batch(
            "CREATE TABLE search_meta (
                singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
                schema_version TEXT NOT NULL,
                search_shard_id TEXT NOT NULL,
                profile_digest TEXT NOT NULL,
                owner_manifest_sha256 TEXT NOT NULL,
                documents_manifest_sha256 TEXT NOT NULL,
                document_count INTEGER NOT NULL CHECK(document_count > 0)
            );
            CREATE TABLE search_documents (
                ordinal INTEGER PRIMARY KEY,
                document_id TEXT NOT NULL UNIQUE COLLATE BINARY,
                owner_metadata_label TEXT NOT NULL COLLATE BINARY
            );
            CREATE INDEX search_documents_label_exact
                ON search_documents(owner_metadata_label COLLATE BINARY);
            CREATE VIRTUAL TABLE search_fts USING fts5(
                document_id UNINDEXED,
                owner_metadata_label,
                tokenize = 'unicode61'
            );",
        )
        .map_err(StoreError::database)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SchemaObject {
    kind: String,
    name: String,
    table_name: String,
    sql: String,
}

fn validate_schema(connection: &Connection) -> StoreResult<()> {
    let observed = schema_shape(connection)?;
    let expected_connection = Connection::open_in_memory().map_err(StoreError::database)?;
    create_schema(&expected_connection)?;
    let expected = schema_shape(&expected_connection)?;
    if observed != expected {
        return Err(integrity_failure());
    }
    Ok(())
}

fn schema_shape(connection: &Connection) -> StoreResult<Vec<SchemaObject>> {
    let mut statement = connection
        .prepare(SCHEMA_INSPECTION_SQL)
        .map_err(StoreError::database)?;
    let rows = statement
        .query_map([], |row| {
            let sql: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
            Ok(SchemaObject {
                kind: row.get(0)?,
                name: row.get(1)?,
                table_name: row.get(2)?,
                sql,
            })
        })
        .map_err(StoreError::database)?;
    let mut objects = Vec::new();
    for row in rows {
        if objects.len() >= MAX_SCHEMA_OBJECTS {
            return Err(integrity_failure());
        }
        objects.push(row.map_err(StoreError::database)?);
    }
    objects.sort();
    Ok(objects)
}

fn validate_database(
    connection: &Connection,
    profile: &SearchStoreProfile,
    shard_id: &str,
    manifest_sha256: &str,
    expected_documents: &[SearchStoreDocument],
    stop: &AtomicBool,
) -> StoreResult<()> {
    check_cancelled(stop)?;
    let metadata = connection
        .query_row(
            "SELECT schema_version, search_shard_id, profile_digest, owner_manifest_sha256, documents_manifest_sha256, document_count FROM search_meta WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )
        .map_err(StoreError::database)?;
    if metadata.0 != PHYSICAL_SCHEMA
        || metadata.1 != shard_id
        || metadata.2 != profile.profile_digest()
        || metadata.3 != manifest_sha256
        || metadata.5 != expected_documents.len() as i64
    {
        return Err(integrity_failure());
    }
    let actual = read_and_validate_documents(connection, profile.limits)?;
    if actual != expected_documents {
        return Err(integrity_failure());
    }
    let actual_manifest = wow_core::canonical_json_bytes(&SearchDocumentManifest {
        schema: PHYSICAL_SCHEMA,
        profile_digest: profile.profile_digest(),
        documents: &actual,
    })
    .map_err(|_| integrity_failure())?;
    if format!("sha256:{}", hex(&Sha256::digest(actual_manifest))) != metadata.4 {
        return Err(integrity_failure());
    }
    validate_fts_rows(connection, &actual, stop)
}

fn read_and_validate_documents(
    connection: &Connection,
    limits: SearchStoreLimits,
) -> StoreResult<Vec<SearchStoreDocument>> {
    let mut statement = connection
        .prepare("SELECT ordinal, document_id, owner_metadata_label FROM search_documents ORDER BY document_id COLLATE BINARY LIMIT ?1")
        .map_err(StoreError::database)?;
    let rows = statement
        .query_map([i64::from(limits.max_documents) + 1], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(StoreError::database)?;
    let mut documents = Vec::new();
    for row in rows {
        if documents.len() >= limits.max_documents as usize {
            return Err(budget_refused());
        }
        let (ordinal, id, label) = row.map_err(StoreError::database)?;
        if id.is_empty()
            || label.is_empty()
            || id.len() > limits.max_document_id_bytes as usize
            || label.len() > limits.max_owner_metadata_label_bytes as usize
            || contains_control(&id)
            || contains_control(&label)
            || ordinal != documents.len() as i64 + 1
        {
            return Err(integrity_failure());
        }
        documents.push(SearchStoreDocument::new(
            id.into_boxed_str(),
            label.into_boxed_str(),
        ));
    }
    if documents.is_empty() {
        return Err(integrity_failure());
    }
    Ok(documents)
}

fn validate_fts_rows(
    connection: &Connection,
    documents: &[SearchStoreDocument],
    stop: &AtomicBool,
) -> StoreResult<()> {
    let mut statement = connection
        .prepare("SELECT rowid, document_id, owner_metadata_label FROM search_fts ORDER BY rowid LIMIT ?1")
        .map_err(StoreError::database)?;
    let rows = statement
        .query_map(
            [i64::try_from(documents.len() + 1).map_err(|_| budget_refused())?],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(StoreError::database)?;
    let mut count = 0_usize;
    for (index, row) in rows.enumerate() {
        check_cancelled(stop)?;
        if index >= documents.len() {
            return Err(integrity_failure());
        }
        let (rowid, id, label) = row.map_err(StoreError::database)?;
        let expected = &documents[index];
        if rowid != index as i64 + 1
            || id != expected.document_id()
            || label != expected.owner_metadata_label()
        {
            return Err(integrity_failure());
        }
        count += 1;
    }
    if count != documents.len() {
        return Err(integrity_failure());
    }
    Ok(())
}

fn check_sqlite_integrity(connection: &Connection) -> StoreResult<()> {
    let mut statement = connection
        .prepare("PRAGMA integrity_check")
        .map_err(StoreError::database)?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(StoreError::database)?;
    let mut seen = 0_usize;
    for row in rows {
        if seen != 0 || row.map_err(StoreError::database)? != "ok" {
            return Err(integrity_failure());
        }
        seen += 1;
    }
    if seen != 1 {
        return Err(integrity_failure());
    }
    Ok(())
}

fn validate_literal(literal: &str, max_bytes: u32) -> StoreResult<()> {
    if literal.is_empty() || literal.len() > max_bytes as usize || contains_control(literal) {
        return Err(budget_refused());
    }
    Ok(())
}

fn quote_fts_literal(literal: &str) -> String {
    let quote_count = literal
        .chars()
        .filter(|character| *character == '"')
        .count();
    let mut query =
        String::with_capacity(literal.len().saturating_add(quote_count).saturating_add(2));
    query.push('"');
    for character in literal.chars() {
        if character == '"' {
            query.push('"');
            query.push('"');
        } else {
            query.push(character);
        }
    }
    query.push('"');
    query
}

fn decode_hex_bounded(value: &str, max_bytes: u64) -> StoreResult<Vec<u8>> {
    if !value.len().is_multiple_of(2) || (value.len() as u64) / 2 > max_bytes {
        return Err(budget_refused());
    }
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() / 2);
    let (pairs, _) = bytes.as_chunks::<2>();
    for pair in pairs {
        let high = hex_value(pair[0]).ok_or_else(integrity_failure)?;
        let low = hex_value(pair[1]).ok_or_else(integrity_failure)?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn contains_control(value: &str) -> bool {
    value.chars().any(char::is_control)
}

fn check_cancelled(stop: &AtomicBool) -> StoreResult<()> {
    if stop.load(AtomicOrdering::Acquire) {
        Err(StoreError::new(
            StoreErrorCode::Cancelled,
            "search shard operation was cancelled",
        ))
    } else {
        Ok(())
    }
}

fn invalid_profile() -> StoreError {
    StoreError::new(
        StoreErrorCode::ConfigurationInvalid,
        "search shard physical profile is invalid",
    )
}

fn budget_refused() -> StoreError {
    StoreError::new(
        StoreErrorCode::BudgetExceeded,
        "search shard request exceeds its supplied bound",
    )
}

fn integrity_failure() -> StoreError {
    StoreError::new(
        StoreErrorCode::IntegrityViolation,
        "search shard artifact failed exact integrity validation",
    )
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    };

    use crate::{
        LeaseId, LeaseRecord, LogicalEpoch, PendingObject, Store, StoreConfiguration, StoreError,
        StoreErrorCode, StoreLimits, WriteBatch,
    };

    use super::{
        SearchStore, SearchStoreDocument, SearchStoreLeaseRequest, SearchStoreLimits,
        SearchStoreProfile, check_cancelled, finish_after_lease,
    };

    type TestResult<T = ()> = Result<T, Box<dyn Error>>;

    fn profile(max_candidates: u32) -> TestResult<SearchStoreProfile> {
        let limits =
            SearchStoreLimits::new(4, 256, 512, 128, max_candidates, 16_384, 256_000, 800_000)?;
        Ok(SearchStoreProfile::new(
            format!("sha256:{}", "a".repeat(64)),
            limits,
        )?)
    }

    fn store() -> TestResult<Store> {
        let limits = StoreLimits::new(1_000_000, 8, 8, 128, 8)?;
        Ok(Store::open_in_memory(StoreConfiguration::new(
            "search-test-v1",
            limits,
        )?)?)
    }

    fn lease(id: &str) -> TestResult<SearchStoreLeaseRequest> {
        Ok(SearchStoreLeaseRequest::new(
            LeaseId::new(id)?,
            "search-test",
            LogicalEpoch::new(10),
        )?)
    }

    #[test]
    fn publication_reopens_read_only_and_runs_exact_and_literal_lanes() -> TestResult {
        let mut store = store()?;
        let shard = SearchStore::build_and_publish(
            &mut store,
            profile(2)?,
            "search-shard:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            &[SearchStoreDocument::new(
                "project-file:state.lua",
                "state.lua",
            )],
            br#"{"schema":"typed-owner-manifest/1"}"#,
            lease("search-lease-one")?,
            &AtomicBool::new(false),
        )?;

        assert_eq!(
            shard.reader().profile_digest(),
            format!("sha256:{}", "a".repeat(64))
        );
        assert_eq!(shard.lease().object_id(), shard.object_id());
        let exact = shard
            .reader()
            .exact_id("project-file:state.lua")?
            .ok_or("exact ID did not resolve")?;
        assert_eq!(exact.owner_metadata_label(), "state.lua");
        assert_eq!(shard.reader().exact_label("state.lua")?.len(), 1);
        let candidates = shard.reader().literal_candidates("state")?;
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].document_id(), "project-file:state.lua");
        assert!(
            shard
                .reader()
                .connection
                .execute("DELETE FROM search_documents", [])
                .is_err()
        );
        let object_id = shard.object_id().clone();
        let while_leased = store.collect_garbage(LogicalEpoch::new(1), 8)?;
        assert!(!while_leased.deleted().contains(&object_id));
        assert!(store.object(&object_id)?.is_some());
        shard.release(&mut store)?;
        let after_release = store.collect_garbage(LogicalEpoch::new(1), 8)?;
        assert!(after_release.deleted().contains(&object_id));
        assert!(store.object(&object_id)?.is_none());
        Ok(())
    }

    fn leased_test_object(store: &mut Store, lease_id: &str) -> TestResult<LeaseRecord> {
        let pending = PendingObject::from_json(
            "wow.search.cleanup-test",
            1,
            &serde_json::json!({"case": lease_id}),
            store.configuration().limits(),
        )?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        store.commit(batch)?;
        Ok(store.acquire_lease(
            LeaseId::new(lease_id)?,
            object_id,
            "search-cleanup-test",
            LogicalEpoch::new(10),
        )?)
    }

    #[test]
    fn post_acquire_cancel_and_reader_failure_release_unexpired_roots() -> TestResult {
        let mut store = store()?;
        let cancelled_lease = leased_test_object(&mut store, "search-cancel-after-lease")?;
        let cancelled_object = cancelled_lease.object_id().clone();
        let stop = AtomicBool::new(false);
        stop.store(true, AtomicOrdering::Release);
        let cancelled = finish_after_lease(
            &mut store,
            cancelled_lease,
            "search-shard:sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                .into(),
            |_, _| {
                check_cancelled(&stop)?;
                Err(StoreError::new(
                    StoreErrorCode::DatabaseUnavailable,
                    "cancellation seam did not fire",
                ))
            },
        )
        .err()
        .ok_or("post-lease cancellation returned a shard")?;
        assert_eq!(cancelled.code(), StoreErrorCode::Cancelled);
        let after_cancel = store.collect_garbage(LogicalEpoch::new(1), 8)?;
        assert_eq!(after_cancel.expired_leases_deleted(), 0);
        assert!(after_cancel.deleted().contains(&cancelled_object));

        let open_failure_lease = leased_test_object(&mut store, "search-open-failure-after-lease")?;
        let open_failure_object = open_failure_lease.object_id().clone();
        let open_failure = finish_after_lease(
            &mut store,
            open_failure_lease,
            "search-shard:sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .into(),
            |_, _| {
                Err(StoreError::new(
                    StoreErrorCode::DatabaseUnavailable,
                    "injected read-only reopen failure",
                ))
            },
        )
        .err()
        .ok_or("post-lease reader failure returned a shard")?;
        assert_eq!(open_failure.code(), StoreErrorCode::DatabaseUnavailable);
        let after_open_failure = store.collect_garbage(LogicalEpoch::new(1), 8)?;
        assert_eq!(after_open_failure.expired_leases_deleted(), 0);
        assert!(after_open_failure.deleted().contains(&open_failure_object));

        let unknown_lease = leased_test_object(&mut store, "search-unknown-cleanup")?;
        let unknown_object = unknown_lease.object_id().clone();
        let unknown_lease_id = unknown_lease.lease_id().as_str().to_owned();
        store.release_lease(unknown_lease.lease_id(), unknown_lease.holder())?;
        let cleanup_unknown = finish_after_lease(
            &mut store,
            unknown_lease,
            "search-shard:sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                .into(),
            |_, _| {
                Err(StoreError::new(
                    StoreErrorCode::DatabaseUnavailable,
                    "injected read-only reopen failure",
                ))
            },
        )
        .err()
        .ok_or("failed lease cleanup returned a shard")?;
        assert_eq!(cleanup_unknown.code(), StoreErrorCode::OutcomeUnknown);
        assert!(cleanup_unknown.message().contains(&unknown_lease_id));
        assert!(cleanup_unknown.message().contains(unknown_object.as_str()));
        assert!(cleanup_unknown.message().contains("expires_after=10"));
        let after_cleanup_conflict = store.collect_garbage(LogicalEpoch::new(1), 8)?;
        assert!(after_cleanup_conflict.deleted().contains(&unknown_object));
        Ok(())
    }

    #[test]
    fn candidate_cap_refuses_the_whole_fts_result() -> TestResult {
        let mut store = store()?;
        let shard = SearchStore::build_and_publish(
            &mut store,
            profile(1)?,
            "search-shard:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            &[
                SearchStoreDocument::new("project-file:alpha.lua", "state.lua"),
                SearchStoreDocument::new("project-file:beta.lua", "state.lua"),
            ],
            br#"{"schema":"typed-owner-manifest/1"}"#,
            lease("search-lease-two")?,
            &AtomicBool::new(false),
        )?;
        let error = shard
            .reader()
            .literal_candidates("state")
            .err()
            .ok_or("candidate cap returned a partial set")?;
        assert_eq!(error.code(), StoreErrorCode::BudgetExceeded);
        Ok(())
    }
}
