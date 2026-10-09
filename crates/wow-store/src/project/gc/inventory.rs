use super::ProjectGcPolicy;
use crate::project::{ProjectStore, model::*, read::*};
use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
use rusqlite::Connection;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};

pub(super) struct Inventory {
    pub generations: BTreeMap<StoreGenerationId, GenerationManifest>,
    pub versions: BTreeMap<PartitionVersionId, PartitionMember>,
    pub protected: BTreeSet<StoreGenerationId>,
    pub operation_versions: BTreeSet<PartitionVersionId>,
    pub state_digest: String,
}

pub(super) fn collect(
    store: &ProjectStore,
    policy: &ProjectGcPolicy,
    stop: &AtomicBool,
) -> StoreResult<Inventory> {
    store.db.ensure_idle()?;
    checkpoint(stop)?;
    let c = store.db.read_connection()?;
    // No semantic/public object authority lies outside these inline partitions.
    let mut versions = BTreeMap::new();
    let mut payload_bytes = 0u64;
    for key in keys(
        &c,
        "SELECT CASE WHEN length(version)<=128 THEN version END FROM partition_versions ORDER BY version LIMIT 8193",
        MAX_VERSIONS as usize,
    )? {
        checkpoint(stop)?;
        let id = PartitionVersionId::parse(key)?;
        let (key, schema, bytes): (String, String, i64) = c.query_row("SELECT CASE WHEN length(logical_key)<=256 THEN logical_key END,CASE WHEN length(schema_id)<=256 THEN schema_id END,byte_length FROM partition_versions WHERE version=?1", [id.as_str()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(StoreError::database)?;
        if bytes < 0
            || bytes as u64 > MAX_RECORD_BYTES as u64
            || !store.db.epoch.catalog.admits(&schema)
        {
            return Err(invalid());
        }
        payload_bytes = payload_bytes
            .checked_add(bytes as u64)
            .filter(|n| *n <= 1024 * 1024 * 1024)
            .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
        let member = PartitionMember {
            key,
            schema,
            byte_length: bytes as usize,
            version: id.clone(),
        };
        read_partition(&c, &member)?;
        versions.insert(id, member);
    }
    let mut generations = BTreeMap::new();
    let mut manifest_bytes = 0usize;
    for key in keys(
        &c,
        "SELECT CASE WHEN length(generation_id)<=128 THEN generation_id END FROM generations ORDER BY generation_id LIMIT 1025",
        MAX_GENERATIONS as usize,
    )? {
        checkpoint(stop)?;
        let id = StoreGenerationId::parse(key)?;
        let manifest = read_manifest(&c, &id, &store.db.epoch)?;
        count_manifest(&manifest, &mut manifest_bytes)?;
        validate_membership(&c, &manifest)?;
        for member in &manifest.members {
            if versions.get(&member.version) != Some(member) {
                return Err(invalid());
            }
        }
        generations.insert(id, manifest);
    }
    let mut validations = Vec::new();
    for key in keys(
        &c,
        "SELECT CASE WHEN length(validation_id)<=128 THEN validation_id END FROM validations ORDER BY validation_id LIMIT 1025",
        MAX_GENERATIONS as usize,
    )? {
        checkpoint(stop)?;
        let id = ValidationId::parse(key)?;
        let generation: String = c.query_row("SELECT CASE WHEN length(generation_id)<=128 THEN generation_id END FROM validations WHERE validation_id=?1",[id.as_str()],|r|r.get(0)).map_err(StoreError::database)?;
        let generation = StoreGenerationId::parse(generation)?;
        let manifest = generations.get(&generation).ok_or_else(invalid)?;
        read_validation(&c, &id, manifest, &store.db.epoch)?;
        validations.push(id);
    }
    let mut history = BTreeMap::new();
    for key in keys(
        &c,
        "SELECT CASE WHEN length(record_id)<=128 THEN record_id END FROM publication_history ORDER BY record_id LIMIT 1025",
        MAX_GENERATIONS as usize,
    )? {
        checkpoint(stop)?;
        let id = CurrentRecordId::parse(key)?;
        let record = read_history(&c, &id, &store.db.epoch)?;
        history.insert(id, record);
    }
    let mut gc_receipts = Vec::new();
    let mut receipt_bytes = 0usize;
    for key in keys(
        &c,
        "SELECT CASE WHEN length(operation_id)<=256 THEN operation_id END FROM gc_operations ORDER BY operation_id LIMIT 4097",
        (MAX_GENERATIONS * 4) as usize,
    )? {
        checkpoint(stop)?;
        let id = OperationId::new(key)?;
        let receipt = store.reconcile_gc(&id)?.ok_or_else(invalid)?;
        receipt_bytes = receipt_bytes
            .checked_add(receipt.canonical_bytes()?.len())
            .filter(|n| *n <= 64 * 1024 * 1024)
            .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
        gc_receipts.push((id, receipt.receipt_digest().to_owned()));
    }
    let current = read_current(&c, &store.db.epoch)?;
    let mut protected = policy.retained_generations().clone();
    let pins = store.retention_roots(stop)?;
    for pin in &pins {
        protected.insert(pin.generation_id().clone());
    }
    for id in store.db.life.leases.borrow().keys() {
        protected.insert(id.clone());
    }
    if let Some(current) = &current {
        protected.insert(current.generation_id.clone());
    }
    for id in &protected {
        if !generations.contains_key(id) {
            return Err(failure(StoreErrorCode::GenerationMissing));
        }
    }
    let mut operations = Vec::new();
    let mut operation_versions = BTreeSet::new();
    for key in keys(
        &c,
        "SELECT CASE WHEN length(operation_id)<=256 THEN operation_id END FROM operations ORDER BY operation_id LIMIT 4097",
        (MAX_GENERATIONS * 4) as usize,
    )? {
        checkpoint(stop)?;
        let id = OperationId::new(key)?;
        let op = read_operation(&c, &id, &store.db.epoch)?.ok_or_else(invalid)?;
        operations.push((id.clone(), op.canonical_digest()?));
        let bytes = super::super::database::blob(&c, "SELECT CASE WHEN length(manifest)<=?2 THEN manifest END FROM operations WHERE operation_id=?1", id.as_str(), 256 * 1024)?.ok_or_else(invalid)?;
        let manifest: GenerationManifest = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        count_manifest(&manifest, &mut manifest_bytes)?;
        if op.release.is_some() {
            continue;
        }
        if generations.contains_key(&manifest.generation_id) {
            protected.insert(manifest.generation_id.clone());
        } else if op.state != PublicationState::Prepared {
            return Err(invalid());
        }
        if let Some(base) = &manifest.expected_current {
            let base = history.get(base).ok_or_else(invalid)?;
            protected.insert(base.generation_id.clone());
        }
        for member in &manifest.members {
            if let Some(actual) = versions.get(&member.version) {
                if actual != member {
                    return Err(invalid());
                }
                operation_versions.insert(member.version.clone());
            } else if op.state != PublicationState::Prepared {
                return Err(invalid());
            }
        }
    }
    let foreign_key_failure: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
            [],
            |r| r.get(0),
        )
        .map_err(StoreError::database)?;
    if foreign_key_failure {
        return Err(invalid());
    }
    checkpoint(stop)?;
    let state_digest = digest(
        "project-gc-state",
        &encode(
            &(
                &current,
                &pins,
                &operations,
                &gc_receipts,
                &generations,
                &versions,
                &validations,
                &history,
            ),
            64 * 1024 * 1024,
        )?,
    );
    Ok(Inventory {
        generations,
        versions,
        protected,
        operation_versions,
        state_digest,
    })
}

fn count_manifest(manifest: &GenerationManifest, total: &mut usize) -> StoreResult<()> {
    *total = total
        .checked_add(encode(manifest, 256 * 1024)?.len())
        .filter(|n| *n <= 64 * 1024 * 1024)
        .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
    Ok(())
}
fn keys(c: &Connection, sql: &'static str, limit: usize) -> StoreResult<Vec<String>> {
    let mut statement = c.prepare(sql).map_err(StoreError::database)?;
    let keys = statement
        .query_map([], |r| r.get(0))
        .map_err(StoreError::database)?
        .collect::<Result<Vec<String>, _>>()
        .map_err(StoreError::database)?;
    if keys.len() > limit {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    Ok(keys)
}
