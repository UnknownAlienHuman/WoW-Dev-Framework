use super::{LiveProjectStore, catalog_for, project_error, store_error};
use crate::{ServiceErrorCode, ServiceResult};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_graph::GraphPartitionSnapshot;
use wow_project::replay::publication::{self, AcquiredProjectPair};
use wow_store::project::{
    CurrentRecordId, MigrationCandidate, MigrationPreparation, ReadyMigration, RegistrySelection,
    ValidatedMigration, VerifiedBackup,
};
use wow_store::{OperationId, StoreErrorCode};

/// Prepare a separate immutable target with complete mapped pins and local Current.
/// No live registry is switched; the original migration/source archive stays exact.
pub fn prepare_live_project_migration(
    migration: &ValidatedMigration,
    root: &Path,
    operation_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<ReadyMigration> {
    let id = OperationId::new(operation_id).map_err(store_error)?;
    let preparation =
        MigrationPreparation::create(migration, root, &id, stop).map_err(store_error)?;
    finish_preparation(migration, preparation, stop)
}

/// Resume one exact durable preparation request with fresh native owner checks.
pub fn resume_live_project_migration_preparation(
    migration: &ValidatedMigration,
    root: &Path,
    operation_id: &str,
    expected_request: &str,
    stop: &AtomicBool,
) -> ServiceResult<ReadyMigration> {
    let id = OperationId::new(operation_id).map_err(store_error)?;
    let preparation = MigrationPreparation::open(migration, root, &id, expected_request, stop)
        .map_err(store_error)?;
    finish_preparation(migration, preparation, stop)
}
fn finish_preparation(
    migration: &ValidatedMigration,
    preparation: MigrationPreparation,
    stop: &AtomicBool,
) -> ServiceResult<ReadyMigration> {
    let mut checks = Vec::new();
    for generation in preparation.target_generations() {
        let read = preparation
            .read_generation(&generation, stop)
            .map_err(store_error)?;
        AcquiredProjectPair::read(&read, stop).map_err(project_error)?;
        checks.push(
            read.owner_validation(&[
                GraphPartitionSnapshot::STORAGE_CHECK,
                publication::STORAGE_CHECK,
            ])
            .map_err(store_error)?,
        );
    }
    preparation
        .finish(migration, checks, stop)
        .map_err(store_error)
}

impl LiveProjectStore {
    /// Build an inactive migration only from the exact guarded live snapshot.
    /// The complete live closure is checked before and after physical staging;
    /// native owner validation follows without selecting the target epoch.
    pub fn migrate_to_new(
        &self,
        source: &VerifiedBackup,
        root: &Path,
        operation_id: &str,
        expected: &RegistrySelection,
        expected_current: Option<&CurrentRecordId>,
        stop: &AtomicBool,
    ) -> ServiceResult<ValidatedMigration> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        let candidate = self
            .store
            .stage_migration_to_new(source, root, &id, expected, expected_current, stop)
            .map_err(store_error)?;
        validate_candidate(candidate, stop)
    }
}

/// Independently export a completed inactive target without changing its baseline.
/// Native owners replay every exported generation; original migration/source
/// metadata remains in the separately retained migration directory.
pub fn export_live_project_migration(
    migration: &ValidatedMigration,
    root: &Path,
    operation_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<VerifiedBackup> {
    let id = OperationId::new(operation_id).map_err(store_error)?;
    let backup = migration
        .export_target(root, &id, stop)
        .map_err(store_error)?;
    for generation in backup.manifest().generations() {
        let read = backup
            .read(
                &wow_store::project::ReadSelector::Exact(generation.clone()),
                stop,
            )
            .map_err(store_error)?;
        AcquiredProjectPair::read(&read, stop).map_err(project_error)?;
    }
    Ok(backup)
}

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
