use super::model::*;
use crate::project::{database::blob, gc, model::*, read::*, retention};
use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
use rusqlite::{Connection, OptionalExtension};
use std::sync::atomic::AtomicBool;

pub(in crate::project) fn inspect(
    c: &Connection,
    epoch: &EpochManifest,
    stop: &AtomicBool,
) -> StoreResult<RecoveryReport> {
    let mut report = RecoveryReport {
        schema: "wow-store/project-recovery/1".into(),
        epoch_id: epoch.epoch_id.clone(),
        physical_profile: epoch.physical_profile.clone(),
        current_state: CurrentState::Unverified,
        current: None,
        operations: Vec::new(),
        coverage: Vec::new(),
        incidents: Vec::new(),
    };
    {
        let mut scope = Scan::new(RecoveryScope::Current, &mut report, stop);
        let result = (|| {
            let current = read_current(c, epoch)?;
            if let Some(current) = &current {
                let manifest = read_manifest(c, &current.generation_id, epoch)?;
                validate_membership(c, &manifest)?;
                for member in &manifest.members {
                    checkpoint(stop)?;
                    read_partition(c, member)?;
                }
            }
            Ok(current)
        })();
        let current = scope.observe(None, result)?;
        let state = scope.state.clone();
        scope.finish();
        match current {
            Some(Some(current)) => {
                report.current_state = CurrentState::Validated;
                report.current = Some(current);
            }
            Some(None) => report.current_state = CurrentState::Absent,
            None if state == ScopeState::Invalid => report.current_state = CurrentState::Corrupt,
            None => {}
        }
    }
    let generation_keys = keys(
        c,
        "SELECT CASE WHEN length(generation_id)<=128 THEN generation_id END FROM generations ORDER BY generation_id LIMIT 1025",
        MAX_GENERATIONS as usize,
    );
    {
        let mut scope = Scan::new(RecoveryScope::Generations, &mut report, stop);
        let mut bytes = 0usize;
        for id in scope
            .observe(None, generation_keys.clone())?
            .unwrap_or_default()
        {
            if scope.state == ScopeState::Incomplete {
                break;
            }
            checkpoint(stop)?;
            scope.observe(
                Some(&id),
                (|| {
                    let manifest = read_manifest(c, &StoreGenerationId::parse(&id)?, epoch)?;
                    count_bytes(
                        &mut bytes,
                        encode(&manifest, 256 * 1024)?.len(),
                        64 * 1024 * 1024,
                    )
                })(),
            )?;
        }
        scope.finish();
    }
    {
        let mut scope = Scan::new(RecoveryScope::Membership, &mut report, stop);
        for id in scope.observe(None, generation_keys)?.unwrap_or_default() {
            if scope.state == ScopeState::Incomplete {
                break;
            }
            checkpoint(stop)?;
            scope.observe(
                Some(&id),
                (|| {
                    let manifest = read_manifest(c, &StoreGenerationId::parse(&id)?, epoch)?;
                validate_membership(c, &manifest)?;
                // Seals are independently hashed in Partitions on this same
                // snapshot. Compare every manifest's complete member descriptor
                // here, including shared seals and operation-free generations.
                for member in &manifest.members {
                    checkpoint(stop)?;
                    let (key, schema, length): (String, String, i64) = c.query_row(
                        "SELECT CASE WHEN length(logical_key)<=256 THEN logical_key END,CASE WHEN length(schema_id)<=256 THEN schema_id END,byte_length FROM partition_versions WHERE version=?1",
                        [member.version.as_str()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
                    ).map_err(StoreError::database)?;
                    if key != member.key || schema != member.schema || length != member.byte_length as i64 { return Err(invalid()); }
                }
                Ok(())
                })(),
            )?;
        }
        scope.finish();
    }
    {
        let mut scope = Scan::new(RecoveryScope::Partitions, &mut report, stop);
        let ids = scope.observe(None, keys(c, "SELECT CASE WHEN length(version)<=128 THEN version END FROM partition_versions ORDER BY version LIMIT 8193", MAX_VERSIONS as usize))?.unwrap_or_default();
        let mut bytes = 0usize;
        for id in ids {
            if scope.state == ScopeState::Incomplete {
                break;
            }
            checkpoint(stop)?;
            scope.observe(Some(&id), (|| {
                let version = PartitionVersionId::parse(&id)?;
                let (key, schema, length): (String, String, i64) = c.query_row("SELECT CASE WHEN length(logical_key)<=256 THEN logical_key END,CASE WHEN length(schema_id)<=256 THEN schema_id END,byte_length FROM partition_versions WHERE version=?1", [&id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(StoreError::database)?;
                let byte_length = usize::try_from(length).map_err(|_| invalid())?;
                if byte_length > MAX_RECORD_BYTES || !epoch.catalog.admits(&schema) { return Err(invalid()); }
                count_bytes(&mut bytes, byte_length, 1024 * 1024 * 1024)?;
                read_partition(c, &PartitionMember { key, schema, byte_length, version }).map(|_| ())
            })())?;
        }
        scope.finish();
    }
    {
        let mut scope = Scan::new(RecoveryScope::Validations, &mut report, stop);
        for id in scope.observe(None, keys(c, "SELECT CASE WHEN length(validation_id)<=128 THEN validation_id END FROM validations ORDER BY validation_id LIMIT 1025", MAX_GENERATIONS as usize))?.unwrap_or_default() {
            if scope.state == ScopeState::Incomplete { break; }
            checkpoint(stop)?;
            scope.observe(Some(&id), (|| {
                let validation = ValidationId::parse(&id)?;
                let generation: String = c.query_row("SELECT CASE WHEN length(generation_id)<=128 THEN generation_id END FROM validations WHERE validation_id=?1", [&id], |r| r.get(0)).map_err(StoreError::database)?;
                let manifest = read_manifest(c, &StoreGenerationId::parse(generation)?, epoch)?;
                read_validation(c, &validation, &manifest, epoch).map(|_| ())
            })())?;
        }
        scope.finish();
    }
    {
        let mut scope = Scan::new(RecoveryScope::History, &mut report, stop);
        for id in scope.observe(None, keys(c, "SELECT CASE WHEN length(record_id)<=128 THEN record_id END FROM publication_history ORDER BY record_id LIMIT 1025", MAX_GENERATIONS as usize))?.unwrap_or_default() {
            if scope.state == ScopeState::Incomplete { break; }
            checkpoint(stop)?;
            scope.observe(Some(&id), (|| read_history(c, &CurrentRecordId::parse(&id)?, epoch))())?;
        }
        scope.finish();
    }
    let current = report.current.clone();
    let mut operations = Vec::new();
    {
        let mut scope = Scan::new(RecoveryScope::Operations, &mut report, stop);
        let mut bytes = 0usize;
        for key in scope.observe(None, keys(c, "SELECT CASE WHEN length(operation_id)<=256 THEN operation_id END FROM operations ORDER BY operation_id LIMIT 4097", (MAX_GENERATIONS * 4) as usize))?.unwrap_or_default() {
            if scope.state == ScopeState::Incomplete { break; }
            checkpoint(stop)?;
            let result = (|| {
                let id = OperationId::new(key.as_str())?;
                let operation = read_operation(c, &id, epoch)?.ok_or_else(invalid)?;
                let raw = blob(c, "SELECT CASE WHEN length(manifest)<=?2 THEN manifest END FROM operations WHERE operation_id=?1", id.as_str(), 256 * 1024)?.ok_or_else(invalid)?;
                count_bytes(&mut bytes, raw.len() + encode(&operation, 65536)?.len(), 8 * 1024 * 1024)?;
                let manifest: GenerationManifest = serde_json::from_slice(&raw).map_err(|_| invalid())?;
                let target_present: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM generations WHERE generation_id=?1)", [operation.generation_id.as_str()], |r| r.get(0)).map_err(StoreError::database)?;
                if operation.release.is_none() {
                    if let Some(base) = &manifest.expected_current { read_history(c, base, epoch)?; }
                    // Prepared can have only some sealed records. Every present seal
                    // must match; absent records are unfinished work, never corruption.
                    for member in &manifest.members {
                        checkpoint(stop)?;
                        let present: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM partition_versions WHERE version=?1)", [member.version.as_str()], |r| r.get(0)).map_err(StoreError::database)?;
                        if present { read_partition(c, member)?; }
                        else if operation.state != PublicationState::Prepared { return Err(invalid()); }
                    }
                }
                let disposition = if operation.release.is_some() { RecoveryDisposition::Released } else {
                    match operation.state {
                        PublicationState::Prepared => RecoveryDisposition::Prepared,
                        PublicationState::PublishedInactive => RecoveryDisposition::Published,
                        PublicationState::ValidatedInactive => RecoveryDisposition::Validated,
                        PublicationState::Activated => RecoveryDisposition::ActivatedReceiptAvailable,
                    }
                };
                Ok(RecoveryOperation {
                    matches_current: current.as_ref().is_some_and(|r| operation.activation.as_ref() == Some(r)),
                    expected_current: manifest.expected_current,
                    operation, disposition, target_present,
                    acknowledgment: AcknowledgmentState::Unknown,
                })
            })();
            if let Some(operation) = scope.observe(Some(&key), result)? { operations.push(operation); }
        }
        scope.finish();
    }
    report.operations = operations;
    if epoch.physical_profile != PHYSICAL_PROFILE {
        let mut scope = Scan::new(RecoveryScope::RetentionRoots, &mut report, stop);
        scope.observe(None, retention::read_roots(c, epoch, stop))?;
        scope.finish();
    } else {
        not_applicable(&mut report, RecoveryScope::RetentionRoots);
    }
    if epoch.physical_profile == GC_PHYSICAL_PROFILE {
        {
            let mut scope = Scan::new(RecoveryScope::GcPolicy, &mut report, stop);
            let result = (|| {
                if let Some(policy) = gc::read_policy(c, epoch)? {
                    for id in policy.retained_generations() {
                        checkpoint(stop)?;
                        read_manifest(c, id, epoch)?;
                    }
                }
                Ok(())
            })();
            scope.observe(None, result)?;
            scope.finish();
        }
        let mut scope = Scan::new(RecoveryScope::GcReceipts, &mut report, stop);
        let mut bytes = 0usize;
        for key in scope.observe(None, keys(c, "SELECT CASE WHEN length(operation_id)<=256 THEN operation_id END FROM gc_operations ORDER BY operation_id LIMIT 4097", (MAX_GENERATIONS * 4) as usize))?.unwrap_or_default() {
            if scope.state == ScopeState::Incomplete { break; }
            checkpoint(stop)?;
            scope.observe(Some(&key), (|| {
                let id = OperationId::new(key.as_str())?;
                let receipt = gc::read_receipt(c, epoch, &id)?.ok_or_else(invalid)?;
                let reused: Option<i64> = c.query_row("SELECT 1 FROM operations WHERE operation_id=?1", [&key], |r| r.get(0)).optional().map_err(StoreError::database)?;
                if reused.is_some() { return Err(invalid()); }
                count_bytes(&mut bytes, receipt.canonical_bytes()?.len(), 64 * 1024 * 1024)
            })())?;
        }
        scope.finish();
    } else {
        not_applicable(&mut report, RecoveryScope::GcPolicy);
        not_applicable(&mut report, RecoveryScope::GcReceipts);
    }
    {
        let mut scope = Scan::new(RecoveryScope::ForeignKeys, &mut report, stop);
        let result = (|| {
            let failed: bool = c
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
                    [],
                    |r| r.get(0),
                )
                .map_err(StoreError::database)?;
            let integrity: String = c
                .query_row("PRAGMA quick_check(1)", [], |r| r.get(0))
                .map_err(StoreError::database)?;
            if failed || integrity != "ok" {
                return Err(invalid());
            }
            Ok(())
        })();
        scope.observe(None, result)?;
        scope.finish();
    }
    checkpoint(stop)?;
    report.canonical_bytes()?;
    Ok(report)
}

struct Scan<'a> {
    scope: RecoveryScope,
    state: ScopeState,
    report: &'a mut RecoveryReport,
    stop: &'a AtomicBool,
}
impl<'a> Scan<'a> {
    fn new(scope: RecoveryScope, report: &'a mut RecoveryReport, stop: &'a AtomicBool) -> Self {
        Self {
            scope,
            report,
            stop,
            state: ScopeState::Validated,
        }
    }
    fn observe<T>(
        &mut self,
        subject: Option<&str>,
        result: StoreResult<T>,
    ) -> StoreResult<Option<T>> {
        checkpoint(self.stop)?;
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.code() == StoreErrorCode::Cancelled => Err(error),
            Err(error) => {
                let incomplete = matches!(
                    error.code(),
                    StoreErrorCode::BudgetExceeded
                        | StoreErrorCode::DatabaseUnavailable
                        | StoreErrorCode::WriterBusy
                        | StoreErrorCode::OutcomeUnknown
                );
                if incomplete {
                    self.state = ScopeState::Incomplete;
                } else if self.state != ScopeState::Incomplete {
                    self.state = ScopeState::Invalid;
                }
                self.report.incidents.push(RecoveryIncident {
                    scope: self.scope.clone(),
                    subject_id: subject.map(str::to_owned),
                    code: error.code(),
                });
                Ok(None)
            }
        }
    }
    fn finish(self) {
        self.report.coverage.push(ScopeCoverage {
            scope: self.scope,
            state: self.state,
        });
    }
}
fn not_applicable(report: &mut RecoveryReport, scope: RecoveryScope) {
    report.coverage.push(ScopeCoverage {
        scope,
        state: ScopeState::NotApplicable,
    });
}
fn count_bytes(total: &mut usize, amount: usize, max: usize) -> StoreResult<()> {
    *total = total
        .checked_add(amount)
        .filter(|n| *n <= max)
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
