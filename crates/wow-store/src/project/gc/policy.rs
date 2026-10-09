use super::super::read::read_manifest;
use super::*;

impl ProjectGcPolicy {
    pub fn canonical_digest(&self) -> StoreResult<String> {
        self.digest()
    }
}
impl ProjectStore {
    pub fn gc_policy(&self) -> StoreResult<Option<ProjectGcPolicy>> {
        require_release(&self.db.epoch)?;
        self.db.ensure_idle()?;
        read_policy(&self.db.connection, &self.db.epoch)
    }
}

pub(in crate::project) fn read_policy(
    c: &rusqlite::Connection,
    epoch: &EpochManifest,
) -> StoreResult<Option<ProjectGcPolicy>> {
    require_release(epoch)?;
    let raw: Option<(String,Vec<u8>)> = c.query_row(
            "SELECT CASE WHEN length(policy_digest)<=128 THEN policy_digest END,CASE WHEN length(record)<=262144 THEN record END FROM gc_policy WHERE id=1",[],|r|Ok((r.get(0)?,r.get(1)?)))
            .optional().map_err(StoreError::database)?;
    let Some((digest, bytes)) = raw else {
        return Ok(None);
    };
    let policy: ProjectGcPolicy = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    policy.validate()?;
    if policy.digest()? != digest || encode(&policy, 256 * 1024)? != bytes {
        return Err(invalid());
    }
    Ok(Some(policy))
}

impl ProjectStore {
    /// Select the exact authoritative policy under expected-current CAS. No
    /// plan selects a policy implicitly, and tightening never deletes data.
    pub fn select_gc_policy(
        &mut self,
        policy: &ProjectGcPolicy,
        expected: Option<&str>,
        stop: &AtomicBool,
    ) -> StoreResult<ProjectGcPolicy> {
        require_release(&self.db.epoch)?;
        self.db.ensure_idle()?;
        checkpoint(stop)?;
        policy.validate()?;
        let old = self.gc_policy()?;
        if old
            .as_ref()
            .map(ProjectGcPolicy::digest)
            .transpose()?
            .as_deref()
            != expected
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        if old.as_ref() == Some(policy) {
            return Ok(policy.clone());
        }
        for id in policy.retained_generations() {
            checkpoint(stop)?;
            read_manifest(&self.db.connection, id, &self.db.epoch)?;
        }
        self.db.write_budget(256 * 1024)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let changed = if let Some(expected) = expected {
            tx.execute(
                "UPDATE gc_policy SET policy_digest=?1,record=?2 WHERE id=1 AND policy_digest=?3",
                params![policy.digest()?, encode(policy, 256 * 1024)?, expected],
            )
        } else {
            tx.execute(
                "INSERT OR IGNORE INTO gc_policy(id,policy_digest,record) VALUES(1,?1,?2)",
                params![policy.digest()?, encode(policy, 256 * 1024)?],
            )
        }
        .map_err(StoreError::database)?;
        if changed != 1 {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        checkpoint(stop)?;
        let committed = tx.commit();
        self.db.ensure_idle()?;
        match self.gc_policy() {
            Ok(Some(actual)) if actual == *policy => Ok(actual),
            Ok(_) if committed.is_ok() => Err(invalid()),
            _ => Err(failure(StoreErrorCode::OutcomeUnknown)),
        }
    }
}
