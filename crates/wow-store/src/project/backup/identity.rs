use crate::project::{
    gc,
    model::*,
    quarantine::archives::{self, QuarantineReference},
    recovery::{self, RecoveryReport, ScopeState},
    retention,
};
use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::atomic::AtomicBool;

/// All persisted semantic identities from one held snapshot. Membership and
/// payloads are already canonically checked by recovery; their IDs bind bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(in crate::project) struct BackupState {
    pub epoch: EpochManifest,
    pub recovery: RecoveryReport,
    pub generations: Vec<StoreGenerationId>,
    pub partitions: Vec<PartitionVersionId>,
    pub validations: Vec<ValidationId>,
    pub history: Vec<CurrentRecordId>,
    pub roots: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    pub gc_receipts: Vec<(OperationId, String)>,
}
impl BackupState {
    pub fn digest(&self) -> StoreResult<String> {
        Ok(digest(
            "project-backup-snapshot",
            &encode(self, 16 * 1024 * 1024)?,
        ))
    }
    pub fn digest_with_quarantines(&self, refs: &[QuarantineReference]) -> StoreResult<String> {
        if refs.is_empty() {
            return self.digest();
        }
        archives::validate_references(refs)?;
        Ok(digest(
            "project-backup-snapshot",
            &encode(
                &("wow-store/project-backup/2", self.digest()?, refs),
                16 * 1024 * 1024,
            )?,
        ))
    }
    pub fn digest_with_authorities(
        &self,
        refs: &[QuarantineReference],
        sources: &[crate::project::source_authority::SourceAuthorityReference],
    ) -> StoreResult<String> {
        if sources.is_empty() {
            return self.digest_with_quarantines(refs);
        }
        crate::project::source_authority::validate_references(sources)?;
        Ok(digest(
            "project-backup-snapshot",
            &encode(
                &(
                    "wow-store/project-backup/3",
                    self.digest_with_quarantines(refs)?,
                    sources,
                ),
                16 * 1024 * 1024,
            )?,
        ))
    }
}
pub(in crate::project) fn capture(
    c: &Connection,
    epoch: &EpochManifest,
    stop: &AtomicBool,
) -> StoreResult<BackupState> {
    checkpoint(stop)?;
    let recovery = recovery::inspect_snapshot(c, epoch, stop)?;
    if recovery
        .coverage()
        .iter()
        .any(|s| s.state() == ScopeState::Invalid)
    {
        return Err(invalid());
    }
    if recovery
        .coverage()
        .iter()
        .any(|s| s.state() == ScopeState::Incomplete)
    {
        let code = recovery
            .incidents()
            .iter()
            .map(|i| i.code())
            .find(|c| {
                matches!(
                    c,
                    StoreErrorCode::BudgetExceeded
                        | StoreErrorCode::DatabaseUnavailable
                        | StoreErrorCode::WriterBusy
                        | StoreErrorCode::OutcomeUnknown,
                )
            })
            .unwrap_or(StoreErrorCode::OutcomeUnknown);
        return Err(failure(code));
    }
    if !recovery.incidents().is_empty() {
        return Err(invalid());
    }
    let generations = keys(
        c,
        "SELECT generation_id FROM generations ORDER BY generation_id LIMIT 1025",
        MAX_GENERATIONS as usize,
        128,
        stop,
    )?
    .into_iter()
    .map(StoreGenerationId::parse)
    .collect::<StoreResult<Vec<_>>>()?;
    let partitions = keys(
        c,
        "SELECT version FROM partition_versions ORDER BY version LIMIT 8193",
        MAX_VERSIONS as usize,
        128,
        stop,
    )?
    .into_iter()
    .map(PartitionVersionId::parse)
    .collect::<StoreResult<Vec<_>>>()?;
    let validations = keys(
        c,
        "SELECT validation_id FROM validations ORDER BY validation_id LIMIT 1025",
        MAX_GENERATIONS as usize,
        128,
        stop,
    )?
    .into_iter()
    .map(ValidationId::parse)
    .collect::<StoreResult<Vec<_>>>()?;
    let history = keys(
        c,
        "SELECT record_id FROM publication_history ORDER BY record_id LIMIT 1025",
        MAX_GENERATIONS as usize,
        128,
        stop,
    )?
    .into_iter()
    .map(CurrentRecordId::parse)
    .collect::<StoreResult<Vec<_>>>()?;
    let roots = if epoch.physical_profile != PHYSICAL_PROFILE {
        retention::read_roots(c, epoch, stop)?
            .into_iter()
            .map(|r| r.pin_digest().to_owned())
            .collect()
    } else {
        Vec::new()
    };
    let (mut policy, mut gc_receipts) = (None, Vec::new());
    if epoch.physical_profile == GC_PHYSICAL_PROFILE {
        policy = gc::read_policy(c, epoch)?
            .as_ref()
            .map(|p| p.canonical_digest())
            .transpose()?;
        for key in keys(
            c,
            "SELECT operation_id FROM gc_operations ORDER BY operation_id LIMIT 4097",
            (MAX_GENERATIONS * 4) as usize,
            256,
            stop,
        )? {
            checkpoint(stop)?;
            let id = OperationId::new(key)?;
            let receipt = gc::read_receipt(c, epoch, &id)?.ok_or_else(invalid)?;
            gc_receipts.push((id, receipt.receipt_digest().to_owned()));
        }
    }
    let state = BackupState {
        epoch: epoch.clone(),
        recovery,
        generations,
        partitions,
        validations,
        history,
        roots,
        policy,
        gc_receipts,
    };
    state.digest()?;
    checkpoint(stop)?;
    Ok(state)
}
fn keys(
    c: &Connection,
    sql: &'static str,
    limit: usize,
    bytes: usize,
    stop: &AtomicBool,
) -> StoreResult<Vec<String>> {
    let mut statement = c.prepare(sql).map_err(StoreError::database)?;
    let mut rows = statement.query([]).map_err(StoreError::database)?;
    let mut result = Vec::new();
    while let Some(row) = rows.next().map_err(StoreError::database)? {
        checkpoint(stop)?;
        if result.len() >= limit {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        // Bound allocation before converting hostile text into an owned String.
        let value = row.get_ref(0).map_err(StoreError::database)?;
        let value = value.as_str().map_err(|_| invalid())?;
        if value.len() > bytes {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        result.push(value.to_owned());
    }
    Ok(result)
}
