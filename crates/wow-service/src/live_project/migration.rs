use super::{catalog_for, project_error, store_error};
use crate::{ServiceErrorCode, ServiceResult};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_graph::GraphPartitionSnapshot;
use wow_project::replay::publication::{self, AcquiredProjectPair};
use wow_store::project::{MigrationCandidate, ValidatedMigration, VerifiedBackup};
use wow_store::{OperationId, StoreErrorCode};

/// Migrate a verified v1/v2 physical snapshot to an unselected v3 target with
/// the original record catalog, then validate every retained native pair.
pub fn migrate_live_project_to_new(
    backup: &VerifiedBackup,
    root: &Path,
    operation_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<ValidatedMigration> {
    let id = OperationId::new(operation_id).map_err(store_error)?;
    let candidate = MigrationCandidate::stage(backup, root, &id, stop).map_err(store_error)?;
    validate_candidate(candidate, stop)
}

/// Reopen the exact staged operation and source snapshot under its original
/// catalog. Completion leaves the target current and root registry absent.
pub fn resume_live_project_migration(
    root: &Path,
    operation_id: &str,
    source_snapshot: &str,
    stop: &AtomicBool,
) -> ServiceResult<ValidatedMigration> {
    let id = OperationId::new(operation_id).map_err(store_error)?;
    for schemas in [
        publication::STORAGE_SCHEMAS,
        publication::STORAGE_SCHEMAS_V3,
        publication::STORAGE_SCHEMAS_V2,
        publication::STORAGE_SCHEMAS_V1,
    ] {
        match MigrationCandidate::open(root, &catalog_for(schemas)?, &id, source_snapshot, stop) {
            Ok(candidate) => return validate_candidate(candidate, stop),
            Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {}
            Err(error) => return Err(store_error(error)),
        }
    }
    Err(super::fail(ServiceErrorCode::IdentityMismatch))
}

fn validate_candidate(
    candidate: MigrationCandidate,
    stop: &AtomicBool,
) -> ServiceResult<ValidatedMigration> {
    let mut checks = Vec::new();
    for id in candidate.target_generations() {
        let read = candidate.read_generation(&id, stop).map_err(store_error)?;
        AcquiredProjectPair::read(&read, stop).map_err(project_error)?;
        checks.push(
            read.owner_validation(&[
                GraphPartitionSnapshot::STORAGE_CHECK,
                publication::STORAGE_CHECK,
            ])
            .map_err(store_error)?,
        );
    }
    candidate.finish(checks, stop).map_err(store_error)
}
