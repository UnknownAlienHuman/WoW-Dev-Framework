//! Explicit release of resumability; original operation evidence is retained.
mod model;
use super::{
    ProjectStore,
    model::*,
    read::{read_operation, save_operation},
};
use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
pub use model::PublicationRelease;
use rusqlite::TransactionBehavior;
use std::sync::atomic::AtomicBool;

impl PublicationOperation {
    pub fn canonical_digest(&self) -> StoreResult<String> {
        Ok(digest("project-original-operation", &encode(self, 65536)?))
    }
}

impl ProjectStore {
    /// Irrevocably relinquish this exact operation's resumability. This does
    /// not change current, erase its receipts or delete its generation.
    pub fn release_publication(
        &mut self,
        id: &OperationId,
        expected_operation_digest: &str,
        held_by: &str,
        stop: &AtomicBool,
    ) -> StoreResult<PublicationOperation> {
        require_release(&self.db.epoch)?;
        self.db.ensure_idle()?;
        checkpoint(stop)?;
        let old = self
            .operation(id)?
            .ok_or_else(|| failure(StoreErrorCode::OperationStateInvalid))?;
        if let Some(receipt) = &old.release {
            if receipt.original_operation_digest() != expected_operation_digest
                || receipt.held_by() != held_by
            {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            return Ok(old);
        }
        if old.canonical_digest()? != expected_operation_digest {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        let mut released = old.clone();
        released.release = Some(PublicationRelease::new(
            self.db.epoch.epoch_id(),
            &old,
            held_by,
        )?);
        self.db.write_budget(65536)?;
        let tx = self
            .db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::database)?;
        let observed = read_operation(&tx, id, &self.db.epoch)?.ok_or_else(invalid)?;
        if observed != old {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        save_operation(&tx, &old, &released)?;
        checkpoint(stop)?;
        let committed = tx.commit();
        self.db.ensure_idle()?;
        match self.operation(id) {
            Ok(Some(actual)) if actual == released => Ok(actual),
            Ok(_) if committed.is_ok() => Err(invalid()),
            _ => Err(failure(StoreErrorCode::OutcomeUnknown)),
        }
    }
}

pub(super) fn require_release(epoch: &EpochManifest) -> StoreResult<()> {
    if epoch.physical_profile() != GC_PHYSICAL_PROFILE {
        return Err(failure(StoreErrorCode::ConfigurationInvalid));
    }
    Ok(())
}

pub(super) fn validate_released(
    op: &PublicationOperation,
    manifest: &GenerationManifest,
    epoch: &EpochManifest,
) -> StoreResult<()> {
    require_release(epoch)?;
    let receipt = op.release.as_ref().ok_or_else(invalid)?;
    let mut original = op.clone();
    original.release = None;
    receipt.validate(epoch.epoch_id(), &original)?;
    // Validate retained original receipt identities without requiring collected
    // generation/history rows. Release itself was admitted from live read-back.
    match op.state {
        PublicationState::Prepared | PublicationState::PublishedInactive
            if op.validation_id.is_none() && op.activation.is_none() => {}
        PublicationState::ValidatedInactive | PublicationState::Activated => {
            let validation = ValidationRecord::new(manifest, epoch.catalog.checks().clone())?;
            if op.validation_id.as_ref() != Some(&validation.validation_id) {
                return Err(invalid());
            }
            if op.state == PublicationState::Activated {
                let expected = CurrentPublication::new(manifest, validation.validation_id)?;
                if op.activation.as_ref() != Some(&expected) {
                    return Err(invalid());
                }
            } else if op.activation.is_some() {
                return Err(invalid());
            }
        }
        _ => return Err(invalid()),
    }
    Ok(())
}
