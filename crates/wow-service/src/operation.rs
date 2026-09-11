use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::identity::validate_identifier;
use crate::{CheckResult, ServiceError, ServiceErrorCode, ServiceResult, StatusResult};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct OperationId(Box<str>);

impl OperationId {
    pub fn new(value: impl Into<Box<str>>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier(&value, "operation_id")?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OperationKind {
    Status,
    Check,
}

#[derive(Debug, Clone)]
pub(crate) enum CompletedResult {
    Status(Arc<StatusResult>),
    Check(Arc<CheckResult>),
}

#[derive(Debug, Clone)]
enum RegistryState {
    InProgress,
    Completed(CompletedResult),
}

#[derive(Debug, Clone)]
struct RegistryEntry {
    request_digest: Box<str>,
    kind: OperationKind,
    state: RegistryState,
}

#[derive(Debug, Clone)]
pub(crate) enum RegistryDecision {
    Started,
    Replay(CompletedResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationRegistryRecord {
    operation_id: Box<str>,
    request_digest: Box<str>,
    kind: OperationKind,
    state: &'static str,
    result_id: Option<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationRegistrySnapshot {
    schema: &'static str,
    records: Vec<OperationRegistryRecord>,
}

impl OperationRegistrySnapshot {
    #[must_use]
    pub fn records(&self) -> &[OperationRegistryRecord] {
        &self.records
    }
}

#[derive(Debug, Default)]
pub(crate) struct OperationRegistry {
    entries: Mutex<BTreeMap<Box<str>, RegistryEntry>>,
}

impl OperationRegistry {
    pub fn begin(
        &self,
        operation_id: &OperationId,
        request_digest: &str,
        kind: OperationKind,
    ) -> ServiceResult<RegistryDecision> {
        let mut entries = self.lock()?;
        if let Some(entry) = entries.get(operation_id.as_str()) {
            if entry.request_digest.as_ref() != request_digest || entry.kind != kind {
                return Err(ServiceError::for_operation(
                    ServiceErrorCode::OperationConflict,
                    "operation ID is already bound to a different canonical request",
                    operation_id.as_str(),
                ));
            }
            return match &entry.state {
                RegistryState::InProgress => Err(ServiceError::for_operation(
                    ServiceErrorCode::OperationBusy,
                    "operation is already in progress",
                    operation_id.as_str(),
                )),
                RegistryState::Completed(result) => Ok(RegistryDecision::Replay(result.clone())),
            };
        }
        entries.insert(
            operation_id.as_str().into(),
            RegistryEntry {
                request_digest: request_digest.into(),
                kind,
                state: RegistryState::InProgress,
            },
        );
        Ok(RegistryDecision::Started)
    }

    pub fn complete(
        &self,
        operation_id: &OperationId,
        request_digest: &str,
        kind: OperationKind,
        result: CompletedResult,
    ) -> ServiceResult<()> {
        let mut entries = self.lock()?;
        let entry = entries.get_mut(operation_id.as_str()).ok_or_else(|| {
            ServiceError::for_operation(
                ServiceErrorCode::InternalContractViolation,
                "operation disappeared before completion",
                operation_id.as_str(),
            )
        })?;
        if entry.request_digest.as_ref() != request_digest
            || entry.kind != kind
            || !matches!(entry.state, RegistryState::InProgress)
        {
            return Err(ServiceError::for_operation(
                ServiceErrorCode::InternalContractViolation,
                "operation completion does not match its registered request",
                operation_id.as_str(),
            ));
        }
        entry.state = RegistryState::Completed(result);
        Ok(())
    }

    pub fn abandon(&self, operation_id: &OperationId, request_digest: &str, kind: OperationKind) {
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };
        if entries.get(operation_id.as_str()).is_some_and(|entry| {
            entry.request_digest.as_ref() == request_digest
                && entry.kind == kind
                && matches!(entry.state, RegistryState::InProgress)
        }) {
            entries.remove(operation_id.as_str());
        }
    }

    pub fn snapshot(&self) -> ServiceResult<OperationRegistrySnapshot> {
        let entries = self.lock()?;
        let records = entries
            .iter()
            .map(|(operation_id, entry)| {
                let (state, result_id) = match &entry.state {
                    RegistryState::InProgress => ("in_progress", None),
                    RegistryState::Completed(CompletedResult::Status(result)) => {
                        ("completed", Some(result.result_id().into()))
                    }
                    RegistryState::Completed(CompletedResult::Check(result)) => {
                        ("completed", Some(result.result_id().into()))
                    }
                };
                OperationRegistryRecord {
                    operation_id: operation_id.clone(),
                    request_digest: entry.request_digest.clone(),
                    kind: entry.kind,
                    state,
                    result_id,
                }
            })
            .collect();
        Ok(OperationRegistrySnapshot {
            schema: "wow-service/operation-registry/1",
            records,
        })
    }

    fn lock(&self) -> ServiceResult<std::sync::MutexGuard<'_, BTreeMap<Box<str>, RegistryEntry>>> {
        self.entries.lock().map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "operation registry lock is poisoned",
            )
        })
    }
}
