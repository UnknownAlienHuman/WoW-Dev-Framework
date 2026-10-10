//! Reconstruct preparation data from the completed migration and frozen source.
use super::{
    super::ValidatedMigration,
    model::{CopyBinding, CurrentPlan, MappedMigrationRoot, PreparationIntent},
};
use crate::project::{
    CurrentPublication, PHYSICAL_PROFILE, PublicationState, RETAINED_PHYSICAL_PROFILE,
    RetentionRoot,
    model::{checkpoint, digest, failure, invalid},
    read, retention,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use std::sync::atomic::AtomicBool;

pub(super) fn build(
    migration: &ValidatedMigration,
    copy: CopyBinding,
    operation: &OperationId,
    stop: &AtomicBool,
) -> StoreResult<PreparationIntent> {
    checkpoint(stop)?;
    migration.verify_completed(stop)?;
    OperationId::new(operation.as_str())?;
    let receipt = migration.receipt();
    let source_manifest = migration.source().manifest();
    let source_epoch = source_manifest.epoch();
    if source_epoch != receipt.source_epoch()
        || source_manifest.snapshot_digest() != receipt.source_snapshot_digest()
    {
        return Err(invalid());
    }
    let source = migration.source().store.db.read_connection()?;
    let source_roots = match source_epoch.physical_profile() {
        PHYSICAL_PROFILE => Vec::new(),
        RETAINED_PHYSICAL_PROFILE => retention::read_roots(&source, source_epoch, stop)?,
        _ => return Err(failure(StoreErrorCode::ConfigurationInvalid)),
    };
    let source_current = read::read_current(&source, source_epoch)?;
    let mut roots = Vec::with_capacity(source_roots.len());
    for source in source_roots {
        checkpoint(stop)?;
        let mapping = migration
            .mappings()
            .iter()
            .find(|mapping| mapping.source_generation() == source.generation_id())
            .ok_or_else(invalid)?;
        let target = RetentionRoot::new(
            receipt.target_epoch().epoch_id().clone(),
            source.root_id().clone(),
            source.kind(),
            mapping.target_generation().clone(),
            source.held_by(),
        )?;
        roots.push(MappedMigrationRoot { source, target });
    }
    roots.sort_by(|left, right| left.target.root_id().cmp(right.target.root_id()));
    let current = match (source_current, receipt.current_mapping()) {
        (None, None) => None,
        (Some(source), Some(current)) if &source == current.source() => {
            checkpoint(stop)?;
            let mapping = migration
                .mappings()
                .iter()
                .find(|mapping| mapping.source_generation() == &source.generation_id)
                .ok_or_else(invalid)?;
            let validation = receipt
                .validations()
                .get(current.target_generation())
                .ok_or_else(invalid)?;
            if mapping.target_generation() != current.target_generation()
                || validation != current.target_validation()
            {
                return Err(invalid());
            }
            let baseline = migration.read_generation(current.target_generation(), stop)?;
            let baseline_operation = migration
                .candidate
                .store
                .operation(mapping.operation_id())?
                .ok_or_else(invalid)?;
            if baseline.manifest().expected_current.is_some()
                || &baseline_operation.operation_id != mapping.operation_id()
                || baseline_operation.request_digest != mapping.request_digest()
                || &baseline_operation.generation_id != current.target_generation()
                || baseline_operation.validation_id.as_ref() != Some(validation)
                || baseline_operation.state != PublicationState::ValidatedInactive
                || baseline_operation.activation.is_some()
                || baseline_operation.release.is_some()
            {
                return Err(invalid());
            }
            Some(CurrentPlan {
                source,
                target: CurrentPublication::new(baseline.manifest(), validation.clone())?,
                operation: mapping.operation_id().clone(),
                request_digest: mapping.request_digest().to_owned(),
            })
        }
        _ => return Err(invalid()),
    };
    let mut generations = migration.target_generations();
    generations.sort();
    let intent = PreparationIntent {
        schema: "wow-store/project-migration-ready-intent/1".into(),
        operation: operation.clone(),
        migration_request: receipt.request_digest().to_owned(),
        migration_receipt_digest: digest("project-migration-receipt", &receipt.bytes()?),
        source_epoch: source_epoch.clone(),
        target_epoch: receipt.target_epoch().clone(),
        source_snapshot: source_manifest.snapshot_digest().to_owned(),
        baseline_snapshot: receipt.target_snapshot_digest().to_owned(),
        copy,
        generations,
        roots,
        current,
        output_operation: super::super::plan::derived_operation(operation, "ready-artifact")?,
    };
    intent.bytes()?;
    checkpoint(stop)?;
    Ok(intent)
}
