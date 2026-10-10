//! Versioned physical selection, independent of semantic epoch identities.
use super::quarantine::archives::{self, QuarantineReference};
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
                None => false,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quarantine_guard: Option<QuarantineReference>,
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
            quarantine_guard: None,
            epoch,
            snapshot_digest,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> StoreResult<()> {
        self.expected.validate()?;
        OperationId::new(self.operation_id.as_str())?;
        let held = self.schema == "wow-store/project-replacement-intent/2";
        if (!held && self.schema != "wow-store/project-replacement-intent/1")
            || self.expected.is_quarantined() != held
            || self.quarantine_guard.is_some() != held
            || (held && self.expected_current.is_some())
            || self.epoch != self.expected.epoch
            || !hashed(&self.snapshot_digest, "project-backup-snapshot")
        {
            return Err(invalid());
        }
        if let Some(reference) = &self.quarantine_guard {
            archives::validate_references(std::slice::from_ref(reference))?;
            if reference.record_digest() != self.expected.digest()
                || self.expected.quarantine.as_ref()
                    != Some(&instance_id(reference.operation_id())?)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn restore(
        operation: OperationId,
        expected: RegistrySelection,
        guard: QuarantineReference,
        snapshot: String,
    ) -> StoreResult<Self> {
        let result = Self {
            schema: "wow-store/project-replacement-intent/2".into(),
            operation_id: operation,
            epoch: expected.epoch.clone(),
            expected,
            expected_current: None,
            quarantine_guard: Some(guard),
            snapshot_digest: snapshot,
        };
        result.validate()?;
        Ok(result)
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_quarantines: Vec<QuarantineReference>,
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
            retained_quarantines: Vec::new(),
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> StoreResult<()> {
        self.intent.validate()?;
        archives::validate_references(&self.retained_quarantines)?;
        let retained = self.schema == "wow-store/project-registry/4";
        if (!retained && self.schema != "wow-store/project-registry/2")
            || retained != !self.retained_quarantines.is_empty()
            || self.epoch.epoch_id != self.intent.epoch
            || self.revision <= self.intent.expected.revision
            || (!retained && self.intent.expected.revision.checked_add(1) != Some(self.revision))
            || self.instance != self.intent.instance()?
            || self.request_digest != self.intent.digest()?
            || !hashed(&self.owner_validation_digest, "project-replacement-owners")
        {
            return Err(invalid());
        }
        if self
            .intent
            .quarantine_guard
            .as_ref()
            .is_some_and(|r| self.retained_quarantines.binary_search(r).is_err())
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
    pub fn with_quarantines(
        epoch: EpochManifest,
        intent: ReplacementIntent,
        owners: String,
        current: Option<CurrentPublication>,
        refs: Vec<QuarantineReference>,
        max_hold_revision: u64,
    ) -> StoreResult<Self> {
        if refs.is_empty() {
            return Self::new(epoch, intent, owners, current);
        }
        let result = Self {
            schema: "wow-store/project-registry/4".into(),
            revision: intent
                .expected
                .revision
                .max(max_hold_revision)
                .checked_add(1)
                .ok_or_else(invalid)?,
            instance: intent.instance()?,
            request_digest: intent.digest()?,
            epoch,
            intent,
            owner_validation_digest: owners,
            activated_current: current,
            retained_quarantines: refs,
        };
        result.validate()?;
        Ok(result)
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
    pub retained_quarantines: Vec<QuarantineReference>,
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
        let previous = read_normal_shallow(catalog, &previous_bytes)?;
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
        let mut refs = previous.retained_quarantines;
        refs.push(QuarantineReference::from_record(&record)?);
        refs.sort();
        let closure = archives::read(
            root,
            catalog,
            &refs,
            &std::sync::atomic::AtomicBool::new(false),
        )?;
        closure.validate_epoch(&record.epoch)?;
        return Ok(AdmittedRegistry {
            epoch: record.epoch.clone(),
            selection: record.selection()?,
            record: previous.record,
            quarantine: Some(record),
            retained_quarantines: refs,
        });
    }
    read_normal(root, catalog, &bytes)
}
pub(super) fn read_normal(
    root: &Path,
    catalog: &RecordCatalog,
    bytes: &[u8],
) -> StoreResult<AdmittedRegistry> {
    let admitted = read_normal_shallow(catalog, bytes)?;
    let holds = archives::read(
        root,
        catalog,
        &admitted.retained_quarantines,
        &std::sync::atomic::AtomicBool::new(false),
    )?;
    holds.validate_epoch(&admitted.epoch)?;
    if holds.max_revision() >= admitted.selection.revision()
        && !admitted.retained_quarantines.is_empty()
    {
        return Err(invalid());
    }
    if let Some(record) = &admitted.record {
        admitted.selection.directory(root, &record.epoch)?;
        let base = record.instance_root(root);
        if read_file(&base.join("replacement-intent.json"), MAX_REGISTRY)?
            != record.intent.bytes()?
            || read_file(&base.join("replacement-record.json"), MAX_REGISTRY)? != bytes
        {
            return Err(invalid());
        }
        let source_exists = match std::fs::symlink_metadata(base.join("replacement-source.json")) {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(_) => return Err(invalid()),
        };
        if record.intent.quarantine_guard.is_none()
            && (source_exists || !record.retained_quarantines.is_empty())
        {
            let source = read_normal_shallow(
                catalog,
                &read_file(&base.join("replacement-source.json"), MAX_REGISTRY)?,
            )?;
            if source.selection != record.intent.expected
                || source.epoch != record.epoch
                || source
                    .retained_quarantines
                    .iter()
                    .any(|r| record.retained_quarantines.binary_search(r).is_err())
            {
                return Err(invalid());
            }
        }
    }
    Ok(admitted)
}
pub(super) fn read_normal_shallow(
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
            retained_quarantines: Vec::new(),
        });
    }
    if let Ok(record) = serde_json::from_slice::<RestoredRegistry>(bytes) {
        record.validate()?;
        database::admit_epoch(&encode(&record.epoch, 65536)?, catalog)?;
        if encode(&record, MAX_REGISTRY)? != bytes {
            return Err(invalid());
        }
        return Ok(AdmittedRegistry {
            selection: RegistrySelection::from_bytes(bytes, &record.epoch, record.revision, None),
            epoch: record.epoch,
            record: None,
            quarantine: None,
            retained_quarantines: record.retained_quarantines,
        });
    }
    let record: RegistryRecord = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    record.validate()?;
    database::admit_epoch(&encode(&record.epoch, 65536)?, catalog)?;
    if record.bytes()? != bytes {
        return Err(invalid());
    }
    let selection = record.selection()?;
    Ok(AdmittedRegistry {
        epoch: record.epoch.clone(),
        selection,
        retained_quarantines: record.retained_quarantines.clone(),
        record: Some(record),
        quarantine: None,
    })
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RestoredRegistry {
    schema: String,
    epoch: EpochManifest,
    operation_id: OperationId,
    snapshot_digest: String,
    owner_validation_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    activated_current: Option<CurrentPublication>,
    revision: u64,
    retained_quarantines: Vec<QuarantineReference>,
}
impl RestoredRegistry {
    fn validate(&self) -> StoreResult<()> {
        OperationId::new(self.operation_id.as_str())?;
        archives::validate_references(&self.retained_quarantines)?;
        if self.schema != "wow-store/project-registry/5"
            || self.retained_quarantines.is_empty()
            || self.revision == 0
            || !hashed(&self.snapshot_digest, "project-backup-snapshot")
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
}
pub(super) fn restored_bytes(
    epoch: &EpochManifest,
    operation: &OperationId,
    snapshot: &str,
    owners: &str,
    current: Option<CurrentPublication>,
    refs: Vec<QuarantineReference>,
    max_hold_revision: u64,
) -> StoreResult<Vec<u8>> {
    if refs.is_empty() {
        return encode(epoch, 65536);
    }
    let record = RestoredRegistry {
        schema: "wow-store/project-registry/5".into(),
        epoch: epoch.clone(),
        operation_id: operation.clone(),
        snapshot_digest: snapshot.into(),
        owner_validation_digest: owners.into(),
        activated_current: current,
        revision: max_hold_revision.checked_add(1).ok_or_else(invalid)?,
        retained_quarantines: refs,
    };
    record.validate()?;
    encode(&record, MAX_REGISTRY)
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
