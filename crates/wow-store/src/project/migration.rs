//! Explicit supported physical migration, stopping at a validated inactive epoch.
mod authority;
mod live;
mod model;
mod plan;
mod ready;
pub use ready::{MappedMigrationRoot, MigrationPreparation, MigrationReadyReceipt, ReadyMigration};
#[cfg(test)]
mod tests;
use super::{
    ProjectStore, ReadSelector, ReadSnapshot, ValidatedRead, VerifiedBackup,
    backup::{self, identity::BackupState},
    database::{self, Database},
    model::*,
    quarantine::archives,
    registry,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use model::{MAX_METADATA, MigrationIntent};
pub use model::{MigrationCurrentMapping, MigrationMapping, MigrationReceipt};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub struct MigrationCandidate {
    store: ProjectStore,
    source: VerifiedBackup,
    intent: MigrationIntent,
    root: PathBuf,
}
/// Held compiled-owner result; a serialized receipt cannot create this value.
pub struct ValidatedMigration {
    candidate: MigrationCandidate,
    receipt: MigrationReceipt,
}
impl MigrationCandidate {
    pub fn stage(
        source: &VerifiedBackup,
        root: &Path,
        operation: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        checkpoint(stop)?;
        OperationId::new(operation.as_str())?;
        source.verify(stop)?;
        if !matches!(
            source.manifest().epoch().physical_profile(),
            PHYSICAL_PROFILE | RETAINED_PHYSICAL_PROFILE
        ) {
            return Err(failure(StoreErrorCode::ConfigurationInvalid));
        }
        let db = Database::create_inactive_with_gc(
            root,
            source.manifest().epoch().owner(),
            source.manifest().epoch().catalog.clone(),
        )?;
        let root = db.root.clone();
        let source_operation = plan::derived_operation(operation, "source-archive")?;
        let source = source.restore_to_new(root.join("source-archive"), &source_operation, stop)?;
        let intent = plan::build(&source, &db.epoch, operation, stop)?;
        // Durable request precedes all target partition/publication writes.
        backup::write_new(&root.join("migration-intent.json"), &intent.bytes()?)?;
        let mut result = Self {
            store: ProjectStore { db },
            source,
            intent,
            root,
        };
        result.prepare_all(stop)?;
        result.verify(stop)?;
        Ok(result)
    }
    /// Reconcile exactly the declared frozen source and original operation.
    /// Only missing work from that immutable request may be resumed.
    pub fn open(
        root: &Path,
        source_catalog: &RecordCatalog,
        operation: &OperationId,
        expected_source_snapshot: &str,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        checkpoint(stop)?;
        let root = database::admitted_root(root)?;
        ensure_unselected(&root)?;
        let bytes = registry::read_file(&root.join("migration-intent.json"), MAX_METADATA)?;
        let intent: MigrationIntent = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        intent.validate()?;
        if intent.bytes()? != bytes
            || &intent.operation_id != operation
            || intent.source_snapshot_digest != expected_source_snapshot
            || intent.source_archive_operation
                != plan::derived_operation(operation, "source-archive")?
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        let source = VerifiedBackup::open(
            root.join("source-archive"),
            source_catalog,
            &intent.source_archive_operation,
            expected_source_snapshot,
            stop,
        )?;
        let computed = plan::build(&source, &intent.target_epoch, operation, stop)?;
        if computed != intent {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        let db = Database::open_inactive(&root, &intent.target_epoch)?;
        let mut result = Self {
            store: ProjectStore { db },
            source,
            intent,
            root,
        };
        // Admit every existing effect before resuming any declared write.
        result.verify_inventory(stop, false)?;
        result.prepare_all(stop)?;
        result.verify(stop)?;
        Ok(result)
    }
    pub fn source(&self) -> &VerifiedBackup {
        &self.source
    }
    pub fn target_epoch(&self) -> &EpochManifest {
        &self.intent.target_epoch
    }
    pub fn mappings(&self) -> &[MigrationMapping] {
        &self.intent.mappings
    }
    pub fn target_generations(&self) -> Vec<StoreGenerationId> {
        plan::representatives(&self.intent)
            .into_iter()
            .map(|m| m.target_generation.clone())
            .collect()
    }
    pub fn read_generation(
        &self,
        generation: &StoreGenerationId,
        stop: &AtomicBool,
    ) -> StoreResult<ReadSnapshot> {
        ensure_unselected(&self.root)?;
        if !self
            .intent
            .mappings
            .iter()
            .any(|m| &m.target_generation == generation)
        {
            return Err(failure(StoreErrorCode::GenerationMissing));
        }
        self.store
            .read(&ReadSelector::Exact(generation.clone()), stop)
    }
    pub fn finish(
        mut self,
        checks: Vec<ValidatedRead>,
        stop: &AtomicBool,
    ) -> StoreResult<ValidatedMigration> {
        self.verify(stop)?;
        let mut capabilities = BTreeMap::new();
        for check in checks {
            checkpoint(stop)?;
            let read = self.read_generation(&check.validation.generation_id, stop)?;
            if check.epoch != self.intent.target_epoch.epoch_id
                || check.validation
                    != ValidationRecord::new(
                        read.manifest(),
                        self.intent.target_epoch.catalog.checks().clone(),
                    )?
                || capabilities
                    .insert(check.validation.generation_id.clone(), check)
                    .is_some()
            {
                return Err(invalid());
            }
        }
        if capabilities.keys().cloned().collect::<BTreeSet<_>>()
            != self.target_generations().into_iter().collect()
        {
            return Err(invalid());
        }
        for mapping in plan::representatives(&self.intent) {
            let check = capabilities
                .remove(&mapping.target_generation)
                .ok_or_else(invalid)?;
            self.store.validate_inactive(
                &mapping.operation_id,
                &mapping.request_digest,
                check,
                stop,
            )?;
        }
        self.verify(stop)?;
        let state = self.capture_target(stop)?;
        let receipt = self.receipt_for(&state)?;
        checkpoint(stop)?;
        write_receipt(&self.root.join("migration-record.json"), &receipt.bytes()?)?;
        let actual = self
            .capture_target(&AtomicBool::new(false))
            .and_then(|s| s.digest())
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if actual != receipt.target_snapshot_digest
            || registry::read_file(&self.root.join("migration-record.json"), MAX_METADATA)?
                != receipt.bytes()?
            || ensure_unselected(&self.root).is_err()
        {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        Ok(ValidatedMigration {
            candidate: self,
            receipt,
        })
    }
    fn receipt_for(&self, state: &BackupState) -> StoreResult<MigrationReceipt> {
        if state.generations != self.target_generations()
            || state.recovery.operations().len() != state.generations.len()
            || state
                .recovery
                .operations()
                .iter()
                .any(|o| o.operation().state != PublicationState::ValidatedInactive)
        {
            return Err(invalid());
        }
        let validations = state
            .recovery
            .operations()
            .iter()
            .map(|o| {
                let operation = o.operation();
                Ok((
                    operation.generation_id.clone(),
                    operation.validation_id.clone().ok_or_else(invalid)?,
                ))
            })
            .collect::<StoreResult<BTreeMap<_, _>>>()?;
        let target_snapshot_digest = state.digest()?;
        let current_mapping = self
            .intent
            .source_current
            .as_ref()
            .map(|current| -> StoreResult<_> {
                let mapping = self
                    .intent
                    .mappings
                    .iter()
                    .find(|m| m.source_generation == current.generation_id)
                    .ok_or_else(invalid)?;
                Ok(MigrationCurrentMapping {
                    source: current.clone(),
                    target_generation: mapping.target_generation.clone(),
                    target_validation: validations
                        .get(&mapping.target_generation)
                        .cloned()
                        .ok_or_else(invalid)?,
                })
            })
            .transpose()?;
        Ok(MigrationReceipt {
            schema: "wow-store/project-migration-record/1".into(),
            request_digest: self.intent.digest()?,
            owner_validation_digest: digest(
                "project-migration-owners",
                &encode(&(&target_snapshot_digest, &validations), MAX_METADATA)?,
            ),
            intent: self.intent.clone(),
            target_snapshot_digest,
            validations,
            current_mapping,
            state: PublicationState::ValidatedInactive,
            acknowledgment: super::AcknowledgmentState::Unknown,
        })
    }
    fn prepare_all(&mut self, stop: &AtomicBool) -> StoreResult<()> {
        ensure_unselected(&self.root)?;
        if self.store.current()?.is_some() {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        for mapping in plan::representatives(&self.intent) {
            let request = plan::request(
                &self.source,
                &self.intent.target_epoch,
                &mapping.source_generation,
                &mapping.operation_id,
                stop,
            )?;
            if request.generation().generation_id != mapping.target_generation
                || request.request_digest() != mapping.request_digest
            {
                return Err(invalid());
            }
            self.store.prepare(&request, stop)?;
        }
        Ok(())
    }
    fn capture_target(&self, stop: &AtomicBool) -> StoreResult<BackupState> {
        ensure_unselected(&self.root)?;
        let read = self.store.db.read_connection()?;
        backup::identity::capture(&read, &self.intent.target_epoch, stop)
    }
    fn verify(&self, stop: &AtomicBool) -> StoreResult<()> {
        self.verify_inventory(stop, true)
    }
    fn verify_inventory(&self, stop: &AtomicBool, complete: bool) -> StoreResult<()> {
        self.source.verify(stop)?;
        let epoch_bytes = encode(&self.intent.target_epoch, 65536)?;
        let epoch_dir = database::epoch_directory(&self.root, &self.intent.target_epoch)?;
        if registry::read_file(&self.root.join("epoch-manifest.json"), 65536)? != epoch_bytes
            || registry::read_file(&epoch_dir.join("epoch-manifest.json"), 65536)? != epoch_bytes
        {
            return Err(invalid());
        }
        if self.intent
            != plan::build(
                &self.source,
                &self.store.db.epoch,
                &self.intent.operation_id,
                stop,
            )?
            || registry::read_file(&self.root.join("migration-intent.json"), MAX_METADATA)?
                != self.intent.bytes()?
        {
            return Err(invalid());
        }
        let connection = self.store.db.read_connection()?;
        let state = backup::identity::capture(&connection, &self.intent.target_epoch, stop)?;
        let expected_ids = self.target_generations();
        let representatives = plan::representatives(&self.intent);
        if (complete && state.generations != expected_ids)
            || state
                .generations
                .iter()
                .any(|id| !expected_ids.contains(id))
            || !state.history.is_empty()
            || !state.roots.is_empty()
            || state.policy.is_some()
            || !state.gc_receipts.is_empty()
            || state.recovery.current().is_some()
            || (complete && state.recovery.operations().len() != representatives.len())
            || state.recovery.operations().iter().any(|o| {
                !representatives
                    .iter()
                    .any(|m| m.operation_id == o.operation().operation_id)
            })
        {
            return Err(invalid());
        }
        let mut partitions = BTreeSet::new();
        let mut validations = BTreeSet::new();
        for mapping in representatives {
            let request = plan::request(
                &self.source,
                &self.intent.target_epoch,
                &mapping.source_generation,
                &mapping.operation_id,
                stop,
            )?;
            let target_present = state.generations.contains(&mapping.target_generation);
            if target_present
                && super::read::read_manifest(
                    &connection,
                    &mapping.target_generation,
                    &self.intent.target_epoch,
                )? != *request.generation()
            {
                return Err(invalid());
            }
            for member in &request.generation().members {
                partitions.insert(member.version.clone());
                if state.partitions.contains(&member.version) {
                    super::read::read_partition(&connection, member)?;
                }
            }
            let operation = state
                .recovery
                .operations()
                .iter()
                .find(|o| o.operation().operation_id == mapping.operation_id);
            let Some(operation) = operation.map(|o| o.operation()) else {
                if complete || target_present {
                    return Err(invalid());
                }
                continue;
            };
            if operation.request_digest != mapping.request_digest
                || operation.generation_id != mapping.target_generation
                || (operation.state == PublicationState::Prepared && (complete || target_present))
                || operation.state == PublicationState::Activated
                || operation.activation.is_some()
                || operation.release.is_some()
            {
                return Err(invalid());
            }
            if let Some(id) = &operation.validation_id {
                validations.insert(id.clone());
            }
        }
        if (complete && state.partitions != partitions.iter().cloned().collect::<Vec<_>>())
            || state.partitions.iter().any(|id| !partitions.contains(id))
            || state.validations != validations.into_iter().collect::<Vec<_>>()
        {
            return Err(invalid());
        }
        // A completed durable record freezes the whole target. It is evidence
        // to reconcile, never permission to recreate missing effects or owners.
        let path = self.root.join("migration-record.json");
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                if registry::read_file(&path, MAX_METADATA)? != self.receipt_for(&state)?.bytes()? {
                    return Err(failure(StoreErrorCode::OperationConflict));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(invalid()),
        }
        Ok(())
    }
}
impl ValidatedMigration {
    /// Export the frozen inactive target into a new independently verified backup.
    /// The migration baseline and its source remain unchanged.
    pub fn export_target(
        &self,
        root: &Path,
        operation: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<VerifiedBackup> {
        self.verify_completed(stop)?;
        let archives = archives::read(
            &self.candidate.root,
            &self.candidate.intent.target_epoch.catalog,
            &[],
            stop,
        )?;
        checkpoint(stop)?;
        let backup = self
            .candidate
            .store
            .backup_to_new_with_archives(root, operation, &archives, stop)?;
        if backup.manifest().epoch() != self.receipt.target_epoch()
            || backup.manifest().snapshot_digest() != self.receipt.target_snapshot_digest()
        {
            return Err(invalid());
        }
        checkpoint(stop)?;
        Ok(backup)
    }
    fn verify_completed(&self, stop: &AtomicBool) -> StoreResult<()> {
        checkpoint(stop)?;
        ensure_unselected(&self.candidate.root)?;
        self.candidate.verify(stop)?;
        if registry::read_file(
            &self.candidate.root.join("migration-record.json"),
            MAX_METADATA,
        )? != self.receipt.bytes()?
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        Ok(())
    }
    pub fn receipt(&self) -> &MigrationReceipt {
        &self.receipt
    }
    pub fn source(&self) -> &VerifiedBackup {
        self.candidate.source()
    }
    pub fn target_epoch(&self) -> &EpochManifest {
        self.candidate.target_epoch()
    }
    pub fn mappings(&self) -> &[MigrationMapping] {
        self.candidate.mappings()
    }
    pub fn target_generations(&self) -> Vec<StoreGenerationId> {
        self.candidate.target_generations()
    }
    pub fn read_generation(
        &self,
        generation: &StoreGenerationId,
        stop: &AtomicBool,
    ) -> StoreResult<ReadSnapshot> {
        self.candidate.read_generation(generation, stop)
    }
}
fn ensure_unselected(root: &Path) -> StoreResult<()> {
    match fs::symlink_metadata(root.join(registry::REGISTRY_FILE)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(failure(StoreErrorCode::CurrentConflict)),
    }
}
fn write_receipt(path: &Path, bytes: &[u8]) -> StoreResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            if registry::read_file(path, MAX_METADATA)? != bytes {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .and_then(|f| f.sync_all())
                .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => backup::write_new(path, bytes),
        Err(_) => Err(invalid()),
    }
}
