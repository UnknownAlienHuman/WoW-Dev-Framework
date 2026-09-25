use super::{
    database::{Database, Lifetime, blob},
    model::*,
};
use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
use rusqlite::{Connection, OptionalExtension, params};
use std::{rc::Rc, sync::atomic::AtomicBool};

#[derive(Debug, Clone)]
pub enum ReadSelector {
    Current,
    Exact(StoreGenerationId),
}

/// A real SQLite read transaction plus process-local generation lease. The
/// writer may activate another generation; this snapshot does not move with it.
pub struct ReadSnapshot {
    connection: Connection,
    manifest: GenerationManifest,
    current_at_acquisition: Option<CurrentPublication>,
    epoch: EpochManifest,
    life: Rc<Lifetime>,
}
impl ReadSnapshot {
    pub(super) fn acquire(
        db: &Database,
        selector: &ReadSelector,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        checkpoint(stop)?;
        if db.life.leases.borrow().values().sum::<usize>() >= MAX_READERS {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        let connection = db.read_connection()?;
        let current = read_current(&connection, &db.epoch)?;
        let id = match selector {
            ReadSelector::Current => {
                &current
                    .as_ref()
                    .ok_or_else(|| failure(StoreErrorCode::GenerationMissing))?
                    .generation_id
            }
            ReadSelector::Exact(id) => id,
        };
        let manifest = read_manifest(&connection, id, &db.epoch)?;
        let mut statement = connection.prepare("SELECT CASE WHEN length(logical_key)<=256 THEN logical_key END,CASE WHEN length(version)<=128 THEN version END FROM membership WHERE generation_id=?1 ORDER BY logical_key LIMIT 257")
            .map_err(StoreError::database)?;
        let members = statement
            .query_map([id.as_str()], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(StoreError::database)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::database)?;
        drop(statement);
        if members.len() != manifest.members.len()
            || members
                .iter()
                .zip(&manifest.members)
                .any(|((key, version), m)| key != &m.key || version != m.version.as_str())
        {
            return Err(invalid());
        }
        // Every member is validated, not just the record eventually requested.
        for member in &manifest.members {
            checkpoint(stop)?;
            read_partition(&connection, member)?;
        }
        checkpoint(stop)?;
        *db.life
            .leases
            .borrow_mut()
            .entry(manifest.generation_id.clone())
            .or_default() += 1;
        Ok(Self {
            connection,
            manifest,
            current_at_acquisition: current,
            epoch: db.epoch.clone(),
            life: Rc::clone(&db.life),
        })
    }
    pub fn manifest(&self) -> &GenerationManifest {
        &self.manifest
    }
    pub fn current_at_acquisition(&self) -> Option<&CurrentPublication> {
        self.current_at_acquisition.as_ref()
    }
    pub fn record(&self, key: &str, stop: &AtomicBool) -> StoreResult<Option<PartitionRecord>> {
        checkpoint(stop)?;
        let Ok(index) = self
            .manifest
            .members
            .binary_search_by(|m| m.key.as_str().cmp(key))
        else {
            return Ok(None);
        };
        let member = self.manifest.members.get(index).ok_or_else(invalid)?;
        let record = read_partition(&self.connection, member)?;
        checkpoint(stop)?;
        Ok(Some(record))
    }
    /// Called by compiled domain adapters only, after their actual owner checks.
    /// Missing/extra checks fail; this is not a JSON-supplied validation flag.
    pub fn owner_validation(
        &self,
        completed_checks: &[&'static str],
    ) -> StoreResult<ValidatedRead> {
        let checks = completed_checks.iter().map(|s| (*s).to_owned()).collect();
        if &checks != self.epoch.catalog.checks() {
            return Err(invalid());
        }
        Ok(ValidatedRead {
            epoch: self.epoch.epoch_id.clone(),
            validation: ValidationRecord::new(&self.manifest, checks)?,
        })
    }
}
impl Drop for ReadSnapshot {
    fn drop(&mut self) {
        let _ = self.connection.execute_batch("ROLLBACK");
        let mut leases = self.life.leases.borrow_mut();
        if let Some(n) = leases.get_mut(&self.manifest.generation_id) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                leases.remove(&self.manifest.generation_id);
            }
        }
    }
}
/// Unserializable read-back capability, bound to the exact validated generation.
pub struct ValidatedRead {
    pub(super) epoch: EpochId,
    pub(super) validation: ValidationRecord,
}

pub(super) fn read_partition(
    c: &Connection,
    member: &PartitionMember,
) -> StoreResult<PartitionRecord> {
    let raw = blob(c,"SELECT CASE WHEN length(payload)<=?2 THEN payload END FROM partition_versions WHERE version=?1",
        member.version.as_str(), MAX_RECORD_BYTES)?.ok_or_else(invalid)?;
    let stored: (String,String,i64) = c.query_row("SELECT CASE WHEN length(logical_key)<=256 THEN logical_key END,CASE WHEN length(schema_id)<=256 THEN schema_id END,byte_length FROM partition_versions WHERE version=?1",
        [member.version.as_str()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(StoreError::database)?;
    if stored.0 != member.key
        || stored.1 != member.schema
        || stored.2 != member.byte_length as i64
        || raw.len() != member.byte_length
    {
        return Err(invalid());
    }
    let record = PartitionRecord::from_bytes(member.key.clone(), member.schema.clone(), raw)?;
    if record.version != member.version {
        return Err(invalid());
    }
    Ok(record)
}
pub(super) fn read_manifest(
    c: &Connection,
    id: &StoreGenerationId,
    epoch: &EpochManifest,
) -> StoreResult<GenerationManifest> {
    let bytes = blob(c,"SELECT CASE WHEN length(manifest)<=?2 THEN manifest END FROM generations WHERE generation_id=?1", id.as_str(),256 * 1024)?
        .ok_or_else(|| failure(StoreErrorCode::GenerationMissing))?;
    let manifest: GenerationManifest = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    manifest.validate(epoch)?;
    if &manifest.generation_id != id || encode(&manifest, 256 * 1024)? != bytes {
        return Err(invalid());
    }
    Ok(manifest)
}
pub(super) fn read_validation(
    c: &Connection,
    id: &ValidationId,
    manifest: &GenerationManifest,
    epoch: &EpochManifest,
) -> StoreResult<ValidationRecord> {
    let bytes = blob(c,"SELECT CASE WHEN length(record)<=?2 THEN record END FROM validations WHERE validation_id=?1",id.as_str(),65536)?.ok_or_else(invalid)?;
    let v: ValidationRecord = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let expected = ValidationRecord::new(manifest, epoch.catalog.checks().clone())?;
    let generation: String = c.query_row("SELECT CASE WHEN length(generation_id)<=128 THEN generation_id END FROM validations WHERE validation_id=?1",[id.as_str()],|r|r.get(0)).map_err(StoreError::database)?;
    if generation != manifest.generation_id.as_str()
        || v != expected
        || &v.validation_id != id
        || bytes != encode(&v, 65536)?
    {
        return Err(invalid());
    }
    Ok(v)
}
pub(super) fn read_current(
    c: &Connection,
    epoch: &EpochManifest,
) -> StoreResult<Option<CurrentPublication>> {
    let id: Option<String> = c.query_row("SELECT CASE WHEN length(record_id)<=128 THEN record_id END FROM current_publication WHERE id=1",[],|r|r.get(0))
        .optional().map_err(StoreError::database)?;
    id.map(|id| read_history(c, &CurrentRecordId::parse(id)?, epoch))
        .transpose()
}
fn read_history(
    c: &Connection,
    id: &CurrentRecordId,
    epoch: &EpochManifest,
) -> StoreResult<CurrentPublication> {
    let bytes = blob(c,"SELECT CASE WHEN length(record)<=?2 THEN record END FROM publication_history WHERE record_id=?1",id.as_str(),65536)?.ok_or_else(invalid)?;
    let record: CurrentPublication = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let manifest = read_manifest(c, &record.generation_id, epoch)?;
    read_validation(c, &record.validation_id, &manifest, epoch)?;
    let expected = CurrentPublication::new(&manifest, record.validation_id.clone())?;
    let keys: (String,String) = c.query_row("SELECT CASE WHEN length(generation_id)<=128 THEN generation_id END,CASE WHEN length(validation_id)<=128 THEN validation_id END FROM publication_history WHERE record_id=?1",[id.as_str()],|r|Ok((r.get(0)?,r.get(1)?))).map_err(StoreError::database)?;
    if keys.0 != record.generation_id.as_str()
        || keys.1 != record.validation_id.as_str()
        || record != expected
        || &record.record_id != id
        || bytes != encode(&record, 65536)?
    {
        return Err(invalid());
    }
    Ok(record)
}
pub(super) fn read_operation(
    c: &Connection,
    id: &OperationId,
    epoch: &EpochManifest,
) -> StoreResult<Option<PublicationOperation>> {
    OperationId::new(id.as_str())?;
    let Some(bytes) = blob(
        c,
        "SELECT CASE WHEN length(record)<=?2 THEN record END FROM operations WHERE operation_id=?1",
        id.as_str(),
        65536,
    )?
    else {
        return Ok(None);
    };
    let op: PublicationOperation = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let manifest_bytes = blob(c,"SELECT CASE WHEN length(manifest)<=?2 THEN manifest END FROM operations WHERE operation_id=?1",id.as_str(),256 * 1024)?.ok_or_else(invalid)?;
    let manifest: GenerationManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|_| invalid())?;
    manifest.validate(epoch)?;
    let stored_digest: String = c.query_row("SELECT CASE WHEN length(request_digest)<=128 THEN request_digest END FROM operations WHERE operation_id=?1",[id.as_str()],|r|r.get(0)).map_err(StoreError::database)?;
    if &op.operation_id != id
        || op.generation_id != manifest.generation_id
        || op.request_digest != stored_digest
        || op.request_digest != digest("project-request", &manifest_bytes)
        || encode(&manifest, 256 * 1024)? != manifest_bytes
        || encode(&op, 65536)? != bytes
    {
        return Err(invalid());
    }
    match op.state {
        PublicationState::Prepared if op.validation_id.is_none() && op.activation.is_none() => {}
        PublicationState::PublishedInactive
            if op.validation_id.is_none() && op.activation.is_none() =>
        {
            if read_manifest(c, &op.generation_id, epoch)? != manifest {
                return Err(invalid());
            }
        }
        PublicationState::ValidatedInactive | PublicationState::Activated => {
            if read_manifest(c, &op.generation_id, epoch)? != manifest {
                return Err(invalid());
            }
            let validation = op.validation_id.as_ref().ok_or_else(invalid)?;
            read_validation(c, validation, &manifest, epoch)?;
            if op.state == PublicationState::Activated {
                let activation = op.activation.as_ref().ok_or_else(invalid)?;
                if activation.generation_id != op.generation_id
                    || &activation.validation_id != validation
                    || read_history(c, &activation.record_id, epoch)? != *activation
                {
                    return Err(invalid());
                }
            } else if op.activation.is_some() {
                return Err(invalid());
            }
        }
        _ => return Err(invalid()),
    }
    Ok(Some(op))
}
pub(super) fn require_base(
    c: &Connection,
    manifest: &GenerationManifest,
    epoch: &EpochManifest,
) -> StoreResult<()> {
    let current = read_current(c, epoch)?;
    if current.as_ref().map(|r| &r.record_id) != manifest.expected_current.as_ref() {
        return Err(failure(StoreErrorCode::CurrentConflict));
    }
    Ok(())
}
pub(super) fn save_operation(
    c: &Connection,
    old: &PublicationOperation,
    new: &PublicationOperation,
) -> StoreResult<()> {
    let changed = c
        .execute(
            "UPDATE operations SET record=?1 WHERE operation_id=?2 AND record=?3",
            params![
                encode(new, 65536)?,
                old.operation_id.as_str(),
                encode(old, 65536)?
            ],
        )
        .map_err(StoreError::database)?;
    if changed != 1 {
        return Err(invalid());
    }
    Ok(())
}
