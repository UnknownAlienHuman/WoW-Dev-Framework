use super::{LiveProjectStore, catalog_for, store_error};
use crate::{ServiceErrorCode, ServiceResult};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_project::replay::publication;
use wow_store::project::{
    CurrentObservation, QuarantineInspection, QuarantineReceipt, QuarantinedStore, RecoveryReport,
    RegistrySelection, ReplacementReceipt, VerifiedBackup,
};
use wow_store::{OperationId, StoreErrorCode};

/// Native physical observation retaining the admitted owner's writer lease.
pub struct LiveProjectQuarantineInspection {
    inspection: QuarantineInspection,
}

impl LiveProjectQuarantineInspection {
    pub fn open(root: &Path, stop: &AtomicBool) -> ServiceResult<Self> {
        for schemas in [
            publication::STORAGE_SCHEMAS,
            publication::STORAGE_SCHEMAS_V7,
            publication::STORAGE_SCHEMAS_V6,
            publication::STORAGE_SCHEMAS_V5,
            publication::STORAGE_SCHEMAS_V4,
            publication::STORAGE_SCHEMAS_V3,
            publication::STORAGE_SCHEMAS_V2,
            publication::STORAGE_SCHEMAS_V1,
        ] {
            match QuarantineInspection::open(root, &catalog_for(schemas)?, stop) {
                Ok(inspection) => return Ok(Self { inspection }),
                Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {}
                Err(error) => return Err(store_error(error)),
            }
        }
        Err(super::fail(ServiceErrorCode::IdentityMismatch))
    }

    pub fn selection(&self) -> &RegistrySelection {
        self.inspection.selection()
    }

    pub fn current(&self) -> &CurrentObservation {
        self.inspection.current()
    }

    pub fn evidence_digest(&self) -> String {
        self.inspection.evidence_digest()
    }

    pub fn quarantine(
        &self,
        operation_id: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<QuarantineReceipt> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.inspection.quarantine(&id, stop).map_err(store_error)
    }
}

/// Native read-only owner for explicitly held physical data.
pub struct QuarantinedLiveProject {
    store: QuarantinedStore,
}

impl QuarantinedLiveProject {
    /// Recover a held instance from an explicit verified target, replaying all
    /// native Project/Graph generations before the guarded registry switch.
    pub fn restore_replace(
        &self,
        backup: &VerifiedBackup,
        operation_id: &str,
        expected: &RegistrySelection,
        stop: &AtomicBool,
    ) -> ServiceResult<(LiveProjectStore, ReplacementReceipt)> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        let candidate = self
            .store
            .stage_restore(backup, &id, expected, stop)
            .map_err(store_error)?;
        let checks = super::recovery::validate_owners(candidate.backup(), stop)?;
        let (store, receipt) = self
            .store
            .activate_restore(candidate, checks, stop)
            .map_err(store_error)?;
        Ok((LiveProjectStore { store }, receipt))
    }

    /// Revalidate and adopt only the exact original staged or selected restore.
    pub fn resume_restore(
        &self,
        operation_id: &str,
        expected: &RegistrySelection,
        snapshot_digest: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<(LiveProjectStore, ReplacementReceipt)> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        let candidate = self
            .store
            .reopen_restore(&id, expected, snapshot_digest, stop)
            .map_err(store_error)?;
        let checks = super::recovery::validate_owners(candidate.backup(), stop)?;
        let (store, receipt) = self
            .store
            .activate_restore(candidate, checks, stop)
            .map_err(store_error)?;
        Ok((LiveProjectStore { store }, receipt))
    }

    pub fn open(root: &Path, stop: &AtomicBool) -> ServiceResult<Self> {
        for schemas in [
            publication::STORAGE_SCHEMAS,
            publication::STORAGE_SCHEMAS_V7,
            publication::STORAGE_SCHEMAS_V6,
            publication::STORAGE_SCHEMAS_V5,
            publication::STORAGE_SCHEMAS_V4,
            publication::STORAGE_SCHEMAS_V3,
            publication::STORAGE_SCHEMAS_V2,
            publication::STORAGE_SCHEMAS_V1,
        ] {
            match QuarantinedStore::open(root, &catalog_for(schemas)?, stop) {
                Ok(store) => return Ok(Self { store }),
                Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {}
                Err(error) => return Err(store_error(error)),
            }
        }
        Err(super::fail(ServiceErrorCode::IdentityMismatch))
    }

    pub fn receipt(&self) -> &QuarantineReceipt {
        self.store.receipt()
    }

    pub fn current_observation(&self, stop: &AtomicBool) -> ServiceResult<CurrentObservation> {
        self.store.current_observation(stop).map_err(store_error)
    }

    pub fn recovery_report(&self, stop: &AtomicBool) -> ServiceResult<RecoveryReport> {
        self.store.recovery_report(stop).map_err(store_error)
    }
}

impl LiveProjectStore {
    pub fn quarantine_inspection(
        &self,
        stop: &AtomicBool,
    ) -> ServiceResult<LiveProjectQuarantineInspection> {
        self.store
            .quarantine_inspection(stop)
            .map(|inspection| LiveProjectQuarantineInspection { inspection })
            .map_err(store_error)
    }

    pub fn quarantine(
        &self,
        operation_id: &str,
        inspection: &LiveProjectQuarantineInspection,
        stop: &AtomicBool,
    ) -> ServiceResult<QuarantineReceipt> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.store
            .quarantine(&id, &inspection.inspection, stop)
            .map_err(store_error)
    }

    pub fn quarantined(&self, stop: &AtomicBool) -> ServiceResult<QuarantinedLiveProject> {
        self.store
            .quarantined(stop)
            .map(|store| QuarantinedLiveProject { store })
            .map_err(store_error)
    }
}
