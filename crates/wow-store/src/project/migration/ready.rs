//! Exact private preparation of mapped pins and target-local Current.
mod inventory;
mod model;
mod plan;
use super::{ValidatedMigration, write_receipt};
use crate::project::{
    AcknowledgmentState, ProjectStore, ReadSelector, ReadSnapshot, StoreGenerationId,
    ValidatedRead, ValidationId, VerifiedBackup, database,
    model::{checkpoint, digest, encode, failure, hashed, invalid},
    registry,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use model::{
    CopyBinding, INTENT_FILE, MAX_METADATA, OUTPUT_DIRECTORY, PreparationIntent, RECORD_FILE,
};
pub use model::{MappedMigrationRoot, MigrationReadyReceipt};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub(super) enum PreparationBody {
    Export(Box<VerifiedBackup>),
    Working(Box<ProjectStore>),
}
/// Held private preparation; persisted intent alone grants no owner authority.
pub struct MigrationPreparation {
    pub(super) body: PreparationBody,
    intent: PreparationIntent,
    root: PathBuf,
}
/// Independently verified immutable target copy, ready for a separate guarded selection.
pub struct ReadyMigration {
    artifact: VerifiedBackup,
    receipt: MigrationReadyReceipt,
}
impl ReadyMigration {
    pub fn artifact(&self) -> &VerifiedBackup {
        &self.artifact
    }
    pub fn receipt(&self) -> &MigrationReadyReceipt {
        &self.receipt
    }
}
impl MigrationPreparation {
    /// Create the preparation copy once; an incomplete existing root is never recopied.
    pub fn create(
        migration: &ValidatedMigration,
        root: &Path,
        operation: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        let copy_operation = super::plan::derived_operation(operation, "preparation-copy")?;
        let export = migration.export_target(root, &copy_operation, stop)?;
        Self::stage(migration, export, operation, stop)
    }
    /// Persist the exact request before any mutable copy handoff. No live epoch is selected.
    pub fn stage(
        migration: &ValidatedMigration,
        export: VerifiedBackup,
        operation: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        migration.verify_completed(stop)?;
        export.verify(stop)?;
        if export.manifest().epoch() != migration.target_epoch()
            || export.manifest().snapshot_digest() != migration.receipt().target_snapshot_digest()
            || export.manifest().current().is_some()
            || !export.manifest().retained_quarantines().is_empty()
        {
            return Err(invalid());
        }
        let intent = plan::build(migration, copy_binding(&export)?, operation, stop)?;
        let root = export.root.clone();
        if present(&root.join(OUTPUT_DIRECTORY))? {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        checkpoint(stop)?;
        write_receipt(&root.join(INTENT_FILE), &intent.bytes()?)?;
        Ok(Self {
            body: PreparationBody::Export(Box::new(export)),
            intent,
            root,
        })
    }
    /// Admit observed state of one original request. Never recopy an interrupted artifact.
    pub fn open(
        migration: &ValidatedMigration,
        root: &Path,
        operation: &OperationId,
        expected_request: &str,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        checkpoint(stop)?;
        let root = database::admitted_root(root)?;
        let bytes = registry::read_file(&root.join(INTENT_FILE), MAX_METADATA)?;
        let intent: PreparationIntent = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if !hashed(expected_request, "project-migration-ready-request")
            || &intent.operation != operation
            || intent.digest()? != expected_request
            || intent.bytes()? != bytes
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        require_intent(migration, &intent, stop)?;
        let body = if present(&root.join(registry::REGISTRY_FILE))? {
            // The original manifest is historical copy metadata only. Mutable SQL
            // is independently admitted against the closed native preparation plan.
            require_copy_metadata(&root, &intent, true)?;
            if registry::read_file(&root.join(registry::REGISTRY_FILE), registry::MAX_REGISTRY)?
                != encode(&intent.target_epoch, 65536)?
            {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            let store = ProjectStore::open(&root, &intent.target_epoch.catalog)?;
            inventory::inspect(&store, migration, &intent, false, stop)?;
            PreparationBody::Working(Box::new(store))
        } else {
            let export = VerifiedBackup::open(
                &root,
                &intent.target_epoch.catalog,
                &intent.copy.operation,
                &intent.baseline_snapshot,
                stop,
            )?;
            if copy_binding(&export)? != intent.copy {
                return Err(invalid());
            }
            inventory::inspect(&export.store, migration, &intent, false, stop)?;
            PreparationBody::Export(Box::new(export))
        };
        Ok(Self { body, intent, root })
    }
    pub fn request_digest(&self) -> StoreResult<String> {
        self.intent.digest()
    }
    pub fn target_generations(&self) -> Vec<StoreGenerationId> {
        self.intent.generations.clone()
    }
    pub fn read_generation(
        &self,
        generation: &StoreGenerationId,
        stop: &AtomicBool,
    ) -> StoreResult<ReadSnapshot> {
        if !self.intent.generations.contains(generation) {
            return Err(failure(StoreErrorCode::GenerationMissing));
        }
        let selector = ReadSelector::Exact(generation.clone());
        match &self.body {
            PreparationBody::Export(backup) => backup.read(&selector, stop),
            PreparationBody::Working(store) => store.read(&selector, stop),
        }
    }
    /// Validate every native owner, reconcile exact pins/Current, then freeze the ready copy.
    /// Output has a fixed confined role; an existing incomplete copy is retained and refused.
    pub fn finish(
        self,
        migration: &ValidatedMigration,
        checks: Vec<ValidatedRead>,
        stop: &AtomicBool,
    ) -> StoreResult<ReadyMigration> {
        require_intent(migration, &self.intent, stop)?;
        if registry::read_file(&self.root.join(INTENT_FILE), MAX_METADATA)?
            != self.intent.bytes()?
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        require_copy_metadata(
            &self.root,
            &self.intent,
            matches!(&self.body, PreparationBody::Working(_)),
        )?;
        let previous = match &self.body {
            PreparationBody::Export(backup) => &backup.store,
            PreparationBody::Working(store) => store.as_ref(),
        };
        inventory::inspect(previous, migration, &self.intent, false, stop)?;
        let validations = inventory::validate_checks(previous, &self.intent, &checks, stop)?;
        let output = self.root.join(OUTPUT_DIRECTORY);
        let existing_output = present(&output)?;
        let existing_artifact = if existing_output {
            if !present(&output.join("backup-manifest.json"))? {
                return Err(failure(StoreErrorCode::OutcomeUnknown));
            }
            let state = inventory::inspect(previous, migration, &self.intent, true, stop)?;
            let artifact = VerifiedBackup::open(
                &output,
                &self.intent.target_epoch.catalog,
                &self.intent.output_operation,
                &state.digest()?,
                stop,
            )?;
            let record_path = output.join(RECORD_FILE);
            if present(&record_path)?
                && registry::read_file(&record_path, MAX_METADATA)?
                    != ready_receipt(&self.intent, &artifact, &validations)?.canonical_bytes()?
            {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            Some(artifact)
        } else {
            None
        };
        let mut store = match self.body {
            PreparationBody::Export(backup) => (*backup).finish_restore(checks, stop)?,
            PreparationBody::Working(store) => *store,
        };
        if !existing_output {
            for root in &self.intent.roots {
                store.put_retention_root(&root.target, stop)?;
            }
            if let Some(current) = &self.intent.current {
                match store.current()? {
                    None => {
                        store.activate(&current.operation, &current.request_digest, stop)?;
                    }
                    Some(actual) if actual == current.target => {}
                    _ => return Err(failure(StoreErrorCode::CurrentConflict)),
                }
            }
        }
        let state = inventory::inspect(&store, migration, &self.intent, true, stop)?;
        let snapshot = state.digest()?;
        let artifact = if let Some(artifact) = existing_artifact {
            artifact
        } else {
            store.backup_to_new(&output, &self.intent.output_operation, stop)?
        };
        if artifact.manifest().epoch() != &self.intent.target_epoch
            || artifact.manifest().snapshot_digest() != snapshot
            || artifact.manifest().current() != self.intent.current.as_ref().map(|p| &p.target)
        {
            return Err(invalid());
        }
        let receipt = ready_receipt(&self.intent, &artifact, &validations)?;
        checkpoint(stop)?;
        write_receipt(&output.join(RECORD_FILE), &receipt.canonical_bytes()?)?;
        // A synced completed record is reconciled independently, even after late cancellation.
        artifact
            .verify(&AtomicBool::new(false))
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if registry::read_file(&output.join(RECORD_FILE), MAX_METADATA)?
            != receipt.canonical_bytes()?
        {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        Ok(ReadyMigration { artifact, receipt })
    }
}
fn ready_receipt(
    intent: &PreparationIntent,
    artifact: &VerifiedBackup,
    validations: &BTreeMap<StoreGenerationId, ValidationId>,
) -> StoreResult<MigrationReadyReceipt> {
    let snapshot = artifact.manifest().snapshot_digest();
    Ok(MigrationReadyReceipt {
        schema: "wow-store/project-migration-ready-record/1".into(),
        request_digest: intent.digest()?,
        artifact: copy_binding(artifact)?,
        owner_validation_digest: digest(
            "project-migration-ready-owners",
            &encode(&(snapshot, validations), MAX_METADATA)?,
        ),
        artifact_snapshot: snapshot.to_owned(),
        intent: intent.clone(),
        acknowledgment: AcknowledgmentState::Unknown,
    })
}
fn require_intent(
    migration: &ValidatedMigration,
    intent: &PreparationIntent,
    stop: &AtomicBool,
) -> StoreResult<()> {
    let computed = plan::build(migration, intent.copy.clone(), &intent.operation, stop)?;
    if &computed != intent {
        return Err(failure(StoreErrorCode::OperationConflict));
    }
    Ok(())
}
fn require_copy_metadata(
    root: &Path,
    intent: &PreparationIntent,
    working: bool,
) -> StoreResult<()> {
    let bytes = registry::read_file(&root.join("backup-manifest.json"), 16 * 1024 * 1024)?;
    if digest("project-migration-copy", &bytes) != intent.copy.manifest_digest {
        return Err(failure(StoreErrorCode::OperationConflict));
    }
    if registry::read_file(&root.join("epoch-manifest.json"), 65536)?
        != encode(&intent.target_epoch, 65536)?
    {
        return Err(invalid());
    }
    let epoch_manifest =
        database::epoch_directory(root, &intent.target_epoch)?.join("epoch-manifest.json");
    if (working || present(&epoch_manifest)?)
        && registry::read_file(&epoch_manifest, 65536)? != encode(&intent.target_epoch, 65536)?
    {
        return Err(invalid());
    }
    Ok(())
}
fn copy_binding(backup: &VerifiedBackup) -> StoreResult<CopyBinding> {
    Ok(CopyBinding {
        operation: backup.manifest().operation_id().clone(),
        request_digest: backup.manifest().request_digest().to_owned(),
        manifest_digest: digest(
            "project-migration-copy",
            &backup.manifest().canonical_bytes()?,
        ),
    })
}
fn present(path: &Path) -> StoreResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(invalid()),
    }
}
