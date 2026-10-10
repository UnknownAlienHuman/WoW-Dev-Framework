//! Confined canonical sidecars for one exact cross-epoch selection request.

use super::model::SelectionIntent;
use crate::project::{
    backup, database,
    model::{failure, invalid},
    registry,
};
use crate::{StoreErrorCode, StoreResult};
use std::{fs, path::Path};

const INTENT_FILE: &str = "migration-selection-intent.json";
const SOURCE_FILE: &str = "migration-selection-source.json";
const MIGRATION_FILE: &str = "migration-selection-migration.json";
const READY_FILE: &str = "migration-selection-ready.json";
const MAX_EVIDENCE: usize = 4 * 1024 * 1024;

pub(in crate::project) fn read_intent(root: &Path) -> StoreResult<SelectionIntent> {
    database::directory(root)?;
    let bytes = registry::read_file(&root.join(INTENT_FILE), registry::MAX_REGISTRY)?;
    let intent: SelectionIntent = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    intent.validate()?;
    if intent.bytes()? != bytes {
        return Err(invalid());
    }
    Ok(intent)
}

pub(in crate::project) fn read(root: &Path, intent: &SelectionIntent) -> StoreResult<()> {
    intent.validate()?;
    if read_intent(root)? != *intent {
        return Err(failure(StoreErrorCode::OperationConflict));
    }
    let source = registry::read_file(&root.join(SOURCE_FILE), registry::MAX_REGISTRY)?;
    let migration = registry::read_file(&root.join(MIGRATION_FILE), MAX_EVIDENCE)?;
    let ready = registry::read_file(&root.join(READY_FILE), MAX_EVIDENCE)?;
    verify_sidecars(intent, &source, &migration, &ready)
}

pub(in crate::project) fn write(
    root: &Path,
    intent: &SelectionIntent,
    source: &[u8],
    migration: &[u8],
    ready: &[u8],
) -> StoreResult<()> {
    database::directory(root)?;
    intent.validate()?;
    let request = intent.bytes()?;
    bounded(&request, registry::MAX_REGISTRY)?;
    verify_sidecars(intent, source, migration, ready)?;
    let files = [
        (INTENT_FILE, request.as_slice(), registry::MAX_REGISTRY),
        (SOURCE_FILE, source, registry::MAX_REGISTRY),
        (MIGRATION_FILE, migration, MAX_EVIDENCE),
        (READY_FILE, ready, MAX_EVIDENCE),
    ];
    // All incoming bindings and every existing sidecar are admitted before effects.
    for (name, bytes, max) in files {
        existing_exact(&root.join(name), bytes, max)?;
    }
    for (name, bytes, max) in files {
        write_exact_or_new(&root.join(name), bytes, max)?;
    }
    Ok(())
}

fn verify_sidecars(
    intent: &SelectionIntent,
    source: &[u8],
    migration: &[u8],
    ready: &[u8],
) -> StoreResult<()> {
    bounded(source, registry::MAX_REGISTRY)?;
    bounded(migration, MAX_EVIDENCE)?;
    bounded(ready, MAX_EVIDENCE)?;
    let selected = registry::read_normal_shallow(&intent.source_epoch.catalog, source)?;
    if selected.epoch != intent.source_epoch || selected.selection != intent.expected {
        return Err(invalid());
    }
    intent.migration_evidence.verify_bytes(migration)?;
    intent.ready_evidence.verify_bytes(ready)
}

fn bounded(bytes: &[u8], max: usize) -> StoreResult<()> {
    if bytes.len() > max {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    Ok(())
}

fn existing_exact(path: &Path, bytes: &[u8], max: usize) -> StoreResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            if registry::read_file(path, max)? != bytes {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(invalid()),
    }
}

fn write_exact_or_new(path: &Path, bytes: &[u8], max: usize) -> StoreResult<()> {
    if existing_exact(path, bytes, max)? {
        // Exact bytes still require a fresh sync after an earlier ambiguous flush.
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .and_then(|file| file.sync_all())
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))
    } else {
        backup::write_new(path, bytes)
    }
}
