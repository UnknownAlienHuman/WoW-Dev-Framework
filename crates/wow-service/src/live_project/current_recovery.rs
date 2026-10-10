//! Physical recovery evidence plus native replay of its exact reported Current.
use super::{LiveProjectStore, project_error, store_error};
use crate::{ServiceErrorCode, ServiceResult};
use serde::Serialize;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_project::{ProjectError, ProjectErrorCode, replay::publication::AcquiredProjectPair};
use wow_store::project::{CurrentState, ReadSelector, RecoveryReport};
use wow_store::{StoreError, StoreErrorCode};

const MAX_OBSERVATION_BYTES: usize = 16 * 1024 * 1024 + 4096;

/// Unchanged physical evidence and one exact publication's domain observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LiveProjectRecoveryObservation {
    schema: &'static str,
    physical: RecoveryReport,
    current_domain: CurrentDomainObservation,
}

impl LiveProjectRecoveryObservation {
    #[must_use]
    pub fn schema(&self) -> &str {
        self.schema
    }

    #[must_use]
    pub fn physical(&self) -> &RecoveryReport {
        &self.physical
    }

    #[must_use]
    pub fn current_domain(&self) -> &CurrentDomainObservation {
        &self.current_domain
    }

    pub fn canonical_bytes(&self) -> ServiceResult<Vec<u8>> {
        let bytes = wow_core::canonical_json_bytes(self)
            .map_err(|_| super::fail(ServiceErrorCode::CanonicalizationFailed))?;
        if bytes.len() > MAX_OBSERVATION_BYTES {
            return Err(super::fail(ServiceErrorCode::BudgetExceeded));
        }
        Ok(bytes)
    }
}

/// Validation applies only to the exact Current retained in the physical report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentDomainObservation {
    Absent,
    Unverified,
    Validated(CurrentDomainIds),
    Failed(CurrentDomainFailure),
    Incomplete(CurrentDomainFailure),
    Cancelled(CurrentDomainFailure),
}

impl CurrentDomainObservation {
    #[must_use]
    pub fn ids(&self) -> Option<&CurrentDomainIds> {
        match self {
            Self::Validated(ids) => Some(ids),
            _ => None,
        }
    }

    #[must_use]
    pub fn failure(&self) -> Option<&CurrentDomainFailure> {
        match self {
            Self::Failed(failure) | Self::Incomplete(failure) | Self::Cancelled(failure) => {
                Some(failure)
            }
            _ => None,
        }
    }
}

/// Identities obtained from the actual replayed native pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CurrentDomainIds {
    publication_set_id: String,
    project_snapshot_id: String,
    analyzer_snapshot_id: String,
    graph_snapshot_id: String,
}

impl CurrentDomainIds {
    #[must_use]
    pub fn publication_set_id(&self) -> &str {
        &self.publication_set_id
    }

    #[must_use]
    pub fn project_snapshot_id(&self) -> &str {
        &self.project_snapshot_id
    }

    #[must_use]
    pub fn analyzer_snapshot_id(&self) -> &str {
        &self.analyzer_snapshot_id
    }

    #[must_use]
    pub fn graph_snapshot_id(&self) -> &str {
        &self.graph_snapshot_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentDomainPhase {
    AcquirePublication,
    Replay,
}

/// A typed failure that retains the already acquired physical evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CurrentDomainFailure {
    phase: CurrentDomainPhase,
    code: ServiceErrorCode,
}

impl CurrentDomainFailure {
    #[must_use]
    pub const fn phase(&self) -> CurrentDomainPhase {
        self.phase
    }

    #[must_use]
    pub const fn code(&self) -> ServiceErrorCode {
        self.code
    }
}

impl LiveProjectStore {
    /// Retain physical recovery evidence, then replay only its reported Current.
    /// Initial scan failures return an error; later failures remain in the report.
    pub fn current_domain_observation(
        &self,
        stop: &AtomicBool,
    ) -> ServiceResult<LiveProjectRecoveryObservation> {
        let physical = self.recovery_report(stop)?;
        let current_domain = self.observe_reported_current(&physical, stop);
        Ok(LiveProjectRecoveryObservation {
            schema: "wow-service/current-domain-recovery/1",
            physical,
            current_domain,
        })
    }

    fn observe_reported_current(
        &self,
        physical: &RecoveryReport,
        stop: &AtomicBool,
    ) -> CurrentDomainObservation {
        match physical.current_state() {
            CurrentState::Absent => return CurrentDomainObservation::Absent,
            CurrentState::Corrupt | CurrentState::Unverified => {
                return CurrentDomainObservation::Unverified;
            }
            CurrentState::Validated => {}
        }
        if stop.load(Ordering::Acquire) {
            return cancelled(CurrentDomainPhase::AcquirePublication);
        }
        let Some(current) = physical.current() else {
            return CurrentDomainObservation::Failed(CurrentDomainFailure {
                phase: CurrentDomainPhase::AcquirePublication,
                code: ServiceErrorCode::IdentityMismatch,
            });
        };
        let read = match self
            .store
            .read(&ReadSelector::Publication(current.record_id.clone()), stop)
        {
            Ok(read) => read,
            Err(error) => return acquisition_outcome(error),
        };
        if &read.manifest().epoch_id != physical.epoch_id()
            || read.manifest().epoch_id != current.epoch_id
            || read.manifest().generation_id != current.generation_id
        {
            return CurrentDomainObservation::Failed(CurrentDomainFailure {
                phase: CurrentDomainPhase::AcquirePublication,
                code: ServiceErrorCode::IdentityMismatch,
            });
        }
        // Publication selected the reported history record. The later observed
        // current_at_acquisition is independent and cannot replace that selection.
        let pair = match AcquiredProjectPair::read(&read, stop) {
            Ok(pair) => pair,
            Err(error) => return replay_outcome(error),
        };
        let ids = CurrentDomainIds {
            publication_set_id: pair.publication_set_id().to_owned(),
            project_snapshot_id: pair.project().snapshot_id().to_owned(),
            analyzer_snapshot_id: pair.project().analyzer_snapshot_id().to_owned(),
            graph_snapshot_id: pair.graph().snapshot().snapshot_id().to_string(),
        };
        if stop.load(Ordering::Acquire) {
            return cancelled(CurrentDomainPhase::Replay);
        }
        CurrentDomainObservation::Validated(ids)
    }
}

/// Observe one exactly admitted store's physical report and reported Current pair.
pub fn recover_current_live_project(
    root: &Path,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectRecoveryObservation> {
    if stop.load(Ordering::Acquire) {
        return Err(super::fail(ServiceErrorCode::Cancelled));
    }
    LiveProjectStore::open(root)?.current_domain_observation(stop)
}

fn cancelled(phase: CurrentDomainPhase) -> CurrentDomainObservation {
    CurrentDomainObservation::Cancelled(CurrentDomainFailure {
        phase,
        code: ServiceErrorCode::Cancelled,
    })
}

fn acquisition_outcome(error: StoreError) -> CurrentDomainObservation {
    let phase = CurrentDomainPhase::AcquirePublication;
    match error.code() {
        StoreErrorCode::Cancelled => cancelled(phase),
        StoreErrorCode::BudgetExceeded
        | StoreErrorCode::ObjectTooLarge
        | StoreErrorCode::DatabaseUnavailable
        | StoreErrorCode::WriterBusy
        | StoreErrorCode::OutcomeUnknown
        | StoreErrorCode::GenerationMissing
        | StoreErrorCode::CurrentConflict
        | StoreErrorCode::Quarantined => {
            CurrentDomainObservation::Incomplete(CurrentDomainFailure {
                phase,
                code: store_error(error).code(),
            })
        }
        _ => CurrentDomainObservation::Failed(CurrentDomainFailure {
            phase,
            code: store_error(error).code(),
        }),
    }
}

fn replay_outcome(error: ProjectError) -> CurrentDomainObservation {
    let phase = CurrentDomainPhase::Replay;
    match error.code() {
        ProjectErrorCode::AnalysisCancelled | ProjectErrorCode::SourceReadCancelled => {
            cancelled(phase)
        }
        ProjectErrorCode::SourceBudgetExceeded | ProjectErrorCode::UpdateBudgetExceeded => {
            CurrentDomainObservation::Incomplete(CurrentDomainFailure {
                phase,
                code: ServiceErrorCode::BudgetExceeded,
            })
        }
        ProjectErrorCode::DeferredCapability => {
            CurrentDomainObservation::Incomplete(CurrentDomainFailure {
                phase,
                code: ServiceErrorCode::OperationNotImplementedForMilestone,
            })
        }
        ProjectErrorCode::StoreReadUnavailable
        | ProjectErrorCode::AnalyzerFailed
        | ProjectErrorCode::SourceReadFailed
        | ProjectErrorCode::MandatoryCapabilityUnavailable => {
            CurrentDomainObservation::Incomplete(CurrentDomainFailure {
                phase,
                code: ServiceErrorCode::ComponentUnavailable,
            })
        }
        _ => CurrentDomainObservation::Failed(CurrentDomainFailure {
            phase,
            code: project_error(error).code(),
        }),
    }
}
