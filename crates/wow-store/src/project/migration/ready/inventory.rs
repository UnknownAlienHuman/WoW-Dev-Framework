//! Closed native inventory checks for an exact READY preparation.

use super::model::PreparationIntent;
use crate::StoreResult;
use crate::project::{
    CurrentPublication, ProjectStore, PublicationState, ReadSelector, StoreGenerationId,
    ValidatedMigration, ValidatedRead, ValidationId,
    backup::identity::{BackupState, capture},
    model::{ValidationRecord, checkpoint, invalid},
    read::{read_history, read_manifest},
    retention,
};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};

pub(super) fn inspect(
    store: &ProjectStore,
    migration: &ValidatedMigration,
    intent: &PreparationIntent,
    complete: bool,
    stop: &AtomicBool,
) -> StoreResult<BackupState> {
    checkpoint(stop)?;
    let _ = intent.bytes()?;
    let baseline_store = &migration.candidate.store;
    let actual_connection = store.db.read_connection()?;
    let baseline_connection = baseline_store.db.read_connection()?;
    let actual = capture(&actual_connection, &store.db.epoch, stop)?;
    let baseline = capture(&baseline_connection, &baseline_store.db.epoch, stop)?;
    let baseline_digest = baseline.digest()?;
    if actual.epoch != baseline.epoch
        || baseline.epoch != intent.target_epoch
        || baseline_digest != intent.baseline_snapshot
        || baseline_digest != migration.receipt().target_snapshot_digest()
        || actual.generations != baseline.generations
        || baseline.generations != intent.generations
        || actual.partitions != baseline.partitions
        || actual.validations != baseline.validations
        || actual.policy.is_some()
        || baseline.policy.is_some()
        || !actual.gc_receipts.is_empty()
        || !baseline.gc_receipts.is_empty()
        || !baseline.roots.is_empty()
        || !baseline.history.is_empty()
        || baseline.recovery.current().is_some()
        || actual.recovery.operations().len() != baseline.recovery.operations().len()
    {
        return Err(invalid());
    }

    for generation in &actual.generations {
        checkpoint(stop)?;
        if read_manifest(&actual_connection, generation, &actual.epoch)?
            != read_manifest(&baseline_connection, generation, &baseline.epoch)?
        {
            return Err(invalid());
        }
    }

    let roots = retention::read_roots(&actual_connection, &actual.epoch, stop)?;
    for root in &roots {
        checkpoint(stop)?;
        let index = intent
            .roots
            .binary_search_by(|planned| planned.target.root_id().cmp(root.root_id()))
            .map_err(|_| invalid())?;
        if root != &intent.roots[index].target {
            return Err(invalid());
        }
    }
    let all_roots = roots.len() == intent.roots.len();
    if complete && !all_roots {
        return Err(invalid());
    }
    if complete && actual.recovery.current() != intent.current.as_ref().map(|plan| &plan.target) {
        return Err(invalid());
    }

    if let Some(plan) = &intent.current {
        let operation = baseline
            .recovery
            .operations()
            .iter()
            .map(|entry| entry.operation())
            .find(|operation| operation.operation_id == plan.operation)
            .ok_or_else(invalid)?;
        let validation = operation.validation_id.as_ref().ok_or_else(invalid)?;
        let manifest = read_manifest(
            &baseline_connection,
            &operation.generation_id,
            &baseline.epoch,
        )?;
        if operation.request_digest != plan.request_digest
            || operation.state != PublicationState::ValidatedInactive
            || CurrentPublication::new(&manifest, validation.clone())? != plan.target
        {
            return Err(invalid());
        }
    }

    let activation = if let Some(current) = actual.recovery.current() {
        let plan = intent.current.as_ref().ok_or_else(invalid)?;
        if current != &plan.target
            || !all_roots
            || actual.history.as_slice() != std::slice::from_ref(&plan.target.record_id)
            || read_history(&actual_connection, &plan.target.record_id, &actual.epoch)?
                != plan.target
        {
            return Err(invalid());
        }
        Some(plan)
    } else {
        if !actual.history.is_empty() {
            return Err(invalid());
        }
        None
    };

    // Native capture orders operations by ID. Equal counts and comparison of
    // every complete record reject missing, substituted and additional keys.
    for (actual_entry, baseline_entry) in actual
        .recovery
        .operations()
        .iter()
        .zip(baseline.recovery.operations())
    {
        checkpoint(stop)?;
        let operation = actual_entry.operation();
        let original = baseline_entry.operation();
        if operation.release.is_some()
            || original.release.is_some()
            || original.state != PublicationState::ValidatedInactive
            || original.activation.is_some()
        {
            return Err(invalid());
        }
        let mut expected = original.clone();
        if let Some(plan) = activation
            && original.operation_id == plan.operation
        {
            expected.state = PublicationState::Activated;
            expected.activation = Some(plan.target.clone());
        }
        if operation != &expected {
            return Err(invalid());
        }
    }

    checkpoint(stop)?;
    Ok(actual)
}

pub(super) fn validate_checks(
    store: &ProjectStore,
    intent: &PreparationIntent,
    checks: &[ValidatedRead],
    stop: &AtomicBool,
) -> StoreResult<BTreeMap<StoreGenerationId, ValidationId>> {
    checkpoint(stop)?;
    let _ = intent.bytes()?;
    if store.db.epoch != intent.target_epoch || checks.len() != intent.generations.len() {
        return Err(invalid());
    }
    let mut validations = BTreeMap::new();
    for check in checks {
        checkpoint(stop)?;
        let generation = &check.validation.generation_id;
        if check.epoch != intent.target_epoch.epoch_id
            || intent.generations.binary_search(generation).is_err()
            || validations.contains_key(generation)
        {
            return Err(invalid());
        }
        let read = store.read(&ReadSelector::Exact(generation.clone()), stop)?;
        if check.validation
            != ValidationRecord::new(
                read.manifest(),
                intent.target_epoch.catalog.checks().clone(),
            )?
        {
            return Err(invalid());
        }
        validations.insert(generation.clone(), check.validation.validation_id.clone());
    }
    if !validations.keys().eq(intent.generations.iter()) {
        return Err(invalid());
    }
    checkpoint(stop)?;
    Ok(validations)
}
