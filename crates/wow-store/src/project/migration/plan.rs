//! Bounded export through the existing typed publication owner.
use super::{
    super::{ReadSelector, VerifiedBackup, model::*},
    model::*,
};
use crate::{OperationId, StoreResult};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};

pub(super) fn derived_operation(
    operation: &OperationId,
    subject: &str,
) -> StoreResult<OperationId> {
    OperationId::new(digest(
        "project-migration-operation",
        &encode(&(operation, subject), 2048)?,
    ))
}
pub(super) fn request(
    source: &VerifiedBackup,
    target: &EpochManifest,
    generation: &StoreGenerationId,
    operation: &OperationId,
    stop: &AtomicBool,
) -> StoreResult<PublicationRequest> {
    checkpoint(stop)?;
    let read = source.read(&ReadSelector::Exact(generation.clone()), stop)?;
    let mut records = Vec::new();
    for member in &read.manifest().members {
        let record = read.record(&member.key, stop)?.ok_or_else(invalid)?;
        if record.version() != &member.version {
            return Err(invalid());
        }
        records.push(record);
    }
    let request = PublicationRequest::new(
        target,
        operation.clone(),
        None,
        read.manifest().bindings.clone(),
        records,
    )?;
    if request.generation().members != read.manifest().members {
        return Err(invalid());
    }
    Ok(request)
}
pub(super) fn build(
    source: &VerifiedBackup,
    target: &EpochManifest,
    operation: &OperationId,
    stop: &AtomicBool,
) -> StoreResult<MigrationIntent> {
    let manifest = source.manifest();
    let mut representatives: BTreeMap<StoreGenerationId, (OperationId, String)> = BTreeMap::new();
    let mut mappings = Vec::new();
    for generation in manifest.generations() {
        checkpoint(stop)?;
        let op = derived_operation(operation, generation.as_str())?;
        let request = request(source, target, generation, &op, stop)?;
        let target_id = request.generation().generation_id.clone();
        let representative = representatives
            .entry(target_id.clone())
            .or_insert_with(|| (op, request.request_digest().to_owned()));
        if representative.1 != request.request_digest() {
            return Err(invalid());
        }
        mappings.push(MigrationMapping {
            source_generation: generation.clone(),
            target_generation: target_id,
            operation_id: representative.0.clone(),
            request_digest: representative.1.clone(),
        });
    }
    let intent = MigrationIntent {
        schema: "wow-store/project-migration-intent/1".into(),
        operation_id: operation.clone(),
        source_archive_operation: manifest.operation_id().clone(),
        source_epoch: manifest.epoch().clone(),
        source_snapshot_digest: manifest.snapshot_digest().into(),
        source_current: manifest.current().cloned(),
        target_epoch: target.clone(),
        mappings,
    };
    intent.validate()?;
    Ok(intent)
}
pub(super) fn representatives(intent: &MigrationIntent) -> Vec<&MigrationMapping> {
    let mut unique = BTreeMap::new();
    for mapping in &intent.mappings {
        unique
            .entry(mapping.target_generation.clone())
            .or_insert(mapping);
    }
    unique.into_values().collect()
}
