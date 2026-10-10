//! Versioned physical selection, independent of semantic epoch identities.
use super::{database, model::*};
use crate::{OperationId, StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub(super) const MAX_REGISTRY: usize = 128 * 1024;
pub(super) const REGISTRY_FILE: &str = "project-store-registry.json";

/// Exact observed outer selector. It is not a generation or a mutable path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrySelection {
    digest: String,
    epoch: EpochId,
    revision: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quarantine: Option<String>,
}
impl RegistrySelection {
    pub fn digest(&self) -> &str {
        &self.digest
    }
    pub fn epoch(&self) -> &EpochId {
        &self.epoch
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub(super) fn from_bytes(
        bytes: &[u8],
        epoch: &EpochManifest,
        revision: u64,
        instance: Option<String>,
    ) -> Self {
        Self {
            digest: digest("project-registry", bytes),
            epoch: epoch.epoch_id.clone(),
            revision,
            instance,
            quarantine: None,
        }
    }
    pub(super) fn validate(&self) -> StoreResult<()> {
        if !hashed(&self.digest, "project-registry")
            || self
                .quarantine
                .as_ref()
                .is_some_and(|id| !instance_valid(id) || self.revision == 0)
            || match &self.instance {
                None => self.revision != 0 && self.quarantine.is_none(),
                Some(id) => self.revision == 0 || !instance_valid(id),
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn is_quarantined(&self) -> bool {
        self.quarantine.is_some()
    }
    pub(super) fn quarantined(&self, bytes: &[u8], revision: u64, id: String) -> Self {
        Self {
            digest: digest("project-registry", bytes),
            epoch: self.epoch.clone(),
            revision,
            instance: self.instance.clone(),
            quarantine: Some(id),
        }
    }
    pub(super) fn instance_root(&self, root: &Path) -> Option<PathBuf> {
        self.instance
            .as_ref()
            .map(|id| root.join("instances").join(id))
    }
    pub(super) fn directory(&self, root: &Path, epoch: &EpochManifest) -> StoreResult<PathBuf> {
        self.validate()?;
        if self.epoch != epoch.epoch_id {
            return Err(invalid());
        }
        let base = match &self.instance {
            Some(id) => {
                database::directory(&root.join("instances"))?;
                let base = root.join("instances").join(id);
                database::directory(&base)?;
                base
            }
            None => root.to_owned(),
        };
        database::directory(&base.join("epochs"))?;
        let dir = database::epoch_directory(&base, epoch)?;
        database::directory(&dir)?;
        Ok(dir)
    }
}

/// Durable guarded intent. No absolute filesystem paths or source payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplacementIntent {
    pub schema: String,
    pub operation_id: OperationId,
    pub expected: RegistrySelection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_current: Option<CurrentRecordId>,
    pub epoch: EpochId,
    pub snapshot_digest: String,
}
impl ReplacementIntent {
    pub fn new(
        operation_id: OperationId,
        expected: RegistrySelection,
        expected_current: Option<CurrentRecordId>,
        epoch: EpochId,
        snapshot_digest: String,
    ) -> StoreResult<Self> {
        let result = Self {
            schema: "wow-store/project-replacement-intent/1".into(),
            operation_id,
            expected,
            expected_current,
            epoch,
            snapshot_digest,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> StoreResult<()> {
        self.expected.validate()?;
        OperationId::new(self.operation_id.as_str())?;
        if self.schema != "wow-store/project-replacement-intent/1"
            || self.expected.is_quarantined()
            || self.epoch != self.expected.epoch
            || !hashed(&self.snapshot_digest, "project-backup-snapshot")
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn bytes(&self) -> StoreResult<Vec<u8>> {
        encode(self, MAX_REGISTRY)
    }
    pub fn digest(&self) -> StoreResult<String> {
        Ok(digest("project-replacement-request", &self.bytes()?))
    }
    pub fn instance(&self) -> StoreResult<String> {
        instance_id(&self.operation_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegistryRecord {
    schema: String,
    pub epoch: EpochManifest,
    pub intent: ReplacementIntent,
    pub revision: u64,
    pub instance: String,
    pub request_digest: String,
    pub owner_validation_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated_current: Option<CurrentPublication>,
}
impl RegistryRecord {
    pub fn new(
        epoch: EpochManifest,
        intent: ReplacementIntent,
        owners: String,
        current: Option<CurrentPublication>,
    ) -> StoreResult<Self> {
        let revision = intent
            .expected
            .revision
            .checked_add(1)
            .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
        let result = Self {
            schema: "wow-store/project-registry/2".into(),
            epoch,
            instance: intent.instance()?,
            request_digest: intent.digest()?,
            intent,
            revision,
            owner_validation_digest: owners,
            activated_current: current,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> StoreResult<()> {
        self.intent.validate()?;
        if self.schema != "wow-store/project-registry/2"
            || self.epoch.epoch_id != self.intent.epoch
            || self.intent.expected.revision.checked_add(1) != Some(self.revision)
            || self.instance != self.intent.instance()?
            || self.request_digest != self.intent.digest()?
            || !hashed(&self.owner_validation_digest, "project-replacement-owners")
        {
            return Err(invalid());
        }
        if let Some(current) = &self.activated_current {
            let bytes = encode(
                &(
                    &current.epoch_id,
                    &current.generation_id,
                    &current.validation_id,
                    current.predecessor.iter().collect::<Vec<_>>(),
                ),
                4096,
            )?;
            if current.epoch_id != self.epoch.epoch_id
                || current.record_id != CurrentRecordId::derive(&bytes)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn bytes(&self) -> StoreResult<Vec<u8>> {
        encode(self, MAX_REGISTRY)
    }
    pub fn selection(&self) -> StoreResult<RegistrySelection> {
        Ok(RegistrySelection::from_bytes(
            &self.bytes()?,
            &self.epoch,
            self.revision,
            Some(self.instance.clone()),
        ))
    }
    pub fn instance_root(&self, root: &Path) -> PathBuf {
        root.join("instances").join(&self.instance)
    }
}

pub(super) struct AdmittedRegistry {
    pub epoch: EpochManifest,
    pub selection: RegistrySelection,
    pub record: Option<RegistryRecord>,
    pub quarantine: Option<super::quarantine::model::QuarantineRecord>,
}
pub(super) fn read(root: &Path, catalog: &RecordCatalog) -> StoreResult<AdmittedRegistry> {
    let bytes = read_file(&root.join(REGISTRY_FILE), MAX_REGISTRY)?;
    if let Ok(record) = serde_json::from_slice::<super::quarantine::model::QuarantineRecord>(&bytes)
    {
        record.validate()?;
        database::admit_epoch(&encode(&record.epoch, 65536)?, catalog)?;
        if record.bytes()? != bytes {
            return Err(invalid());
        }
        database::directory(&root.join("quarantines"))?;
        let archive = record.archive(root)?;
        database::directory(&archive)?;
        let previous_bytes = read_file(&archive.join("selection.json"), MAX_REGISTRY)?;
        // Archived authority is a normal selector only: no recursive history decoder.
        let previous = read_normal(root, catalog, &previous_bytes)?;
        let evidence = read_file(
            &archive.join("evidence.json"),
            super::quarantine::model::MAX_EVIDENCE,
        )?;
        if previous.selection != record.previous
            || previous.epoch != record.epoch
            || evidence.len() != record.evidence_length
            || digest("project-quarantine-evidence", &evidence) != record.evidence_digest
            || read_file(&archive.join("record.json"), MAX_REGISTRY)? != bytes
        {
            return Err(invalid());
        }
        return Ok(AdmittedRegistry {
            epoch: record.epoch.clone(),
            selection: record.selection()?,
            record: previous.record,
            quarantine: Some(record),
        });
    }
    read_normal(root, catalog, &bytes)
}
pub(super) fn read_normal(
    root: &Path,
    catalog: &RecordCatalog,
    bytes: &[u8],
) -> StoreResult<AdmittedRegistry> {
    // Each branch reconstructs canonical bytes; legacy manifests are unchanged.
    if let Ok(epoch) = database::admit_epoch(bytes, catalog) {
        return Ok(AdmittedRegistry {
            selection: RegistrySelection::from_bytes(bytes, &epoch, 0, None),
            epoch,
            record: None,
            quarantine: None,
        });
    }
    let record: RegistryRecord = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    record.validate()?;
    database::admit_epoch(&encode(&record.epoch, 65536)?, catalog)?;
    if record.bytes()? != bytes {
        return Err(invalid());
    }
    let selection = record.selection()?;
    selection.directory(root, &record.epoch)?;
    let base = record.instance_root(root);
    if read_file(&base.join("replacement-intent.json"), MAX_REGISTRY)? != record.intent.bytes()?
        || read_file(&base.join("replacement-record.json"), MAX_REGISTRY)? != bytes
    {
        return Err(invalid());
    }
    Ok(AdmittedRegistry {
        epoch: record.epoch.clone(),
        selection,
        record: Some(record),
        quarantine: None,
    })
}
pub(super) fn read_file(path: &Path, max: usize) -> StoreResult<Vec<u8>> {
    database::regular(path, max as u64)?;
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(max as u64 + 1).read_to_end(&mut bytes))
        .map_err(|_| invalid())?;
    if bytes.len() > max {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    Ok(bytes)
}
pub(super) fn instance_id(operation: &OperationId) -> StoreResult<String> {
    Ok(digest("project-instance", &encode(operation, 1024)?)
        .strip_prefix("project-instance:sha256:")
        .ok_or_else(invalid)?
        .to_owned())
}
fn instance_valid(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
