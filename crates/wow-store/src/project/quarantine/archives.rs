//! Finite, flat, portable hold authority. Historical SQL bodies are not inputs.
use super::{
    super::{database, model::*, registry, replacement},
    model::QuarantineRecord,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path, sync::atomic::AtomicBool};

pub(in crate::project) const MAX_HOLDS: usize = 32;
const MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuarantineReference {
    operation_id: OperationId,
    record_digest: String,
}
impl QuarantineReference {
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn record_digest(&self) -> &str {
        &self.record_digest
    }
    pub(in crate::project) fn from_record(record: &QuarantineRecord) -> StoreResult<Self> {
        Ok(Self {
            operation_id: record.operation_id.clone(),
            record_digest: digest("project-registry", &record.bytes()?),
        })
    }
    fn validate(&self) -> StoreResult<()> {
        OperationId::new(self.operation_id.as_str())?;
        if !hashed(&self.record_digest, "project-registry") {
            return Err(invalid());
        }
        Ok(())
    }
}
struct Archive {
    reference: QuarantineReference,
    record: QuarantineRecord,
    selection: Vec<u8>,
    evidence: Vec<u8>,
}
pub(in crate::project) struct ArchiveSet {
    references: Vec<QuarantineReference>,
    archives: Vec<Archive>,
}
impl ArchiveSet {
    pub fn references(&self) -> &[QuarantineReference] {
        &self.references
    }
    pub fn max_revision(&self) -> u64 {
        self.archives
            .iter()
            .map(|a| a.record.revision)
            .max()
            .unwrap_or(0)
    }
    pub fn validate_epoch(&self, epoch: &EpochManifest) -> StoreResult<()> {
        if self.archives.iter().any(|a| &a.record.epoch != epoch) {
            return Err(invalid());
        }
        Ok(())
    }
    pub(super) fn admit_hold(
        &self,
        record: &QuarantineRecord,
        selection_bytes: usize,
        evidence_bytes: usize,
    ) -> StoreResult<()> {
        if self
            .references
            .iter()
            .any(|r| r.operation_id == record.operation_id)
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        if self.references.len() >= MAX_HOLDS {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        let record_length = record.bytes()?.len();
        self.byte_length()?
            .checked_add(selection_bytes)
            .and_then(|n| n.checked_add(evidence_bytes))
            .and_then(|n| n.checked_add(record_length))
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
        Ok(())
    }
    pub(in crate::project) fn byte_length(&self) -> StoreResult<usize> {
        self.archives.iter().try_fold(0usize, |n, a| {
            let record_length = a.record.bytes()?.len();
            n.checked_add(a.selection.len())
                .and_then(|n| n.checked_add(a.evidence.len()))
                .and_then(|n| n.checked_add(record_length))
                .filter(|n| *n <= MAX_BYTES)
                .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))
        })
    }
    pub fn merge(self, other: Self) -> StoreResult<Self> {
        let mut entries = BTreeMap::new();
        for archive in self.archives.into_iter().chain(other.archives) {
            if let Some(previous) = entries.get(archive.reference.operation_id()) {
                let previous: &Archive = previous;
                if previous.reference != archive.reference
                    || previous.selection != archive.selection
                    || previous.evidence != archive.evidence
                {
                    return Err(failure(StoreErrorCode::OperationConflict));
                }
            } else {
                entries.insert(archive.reference.operation_id.clone(), archive);
            }
        }
        let result = Self {
            references: entries.values().map(|a| a.reference.clone()).collect(),
            archives: entries.into_values().collect(),
        };
        validate_references(&result.references)?;
        result.byte_length()?;
        Ok(result)
    }
    pub(in crate::project) fn hold_identities(&self) -> Vec<(EpochId, QuarantineReference)> {
        self.archives
            .iter()
            .map(|a| (a.record.epoch.epoch_id().clone(), a.reference.clone()))
            .collect()
    }
    pub(in crate::project) fn required_authorities(
        &self,
    ) -> StoreResult<Vec<super::super::source_authority::SourceAuthorityReference>> {
        let mut refs = Vec::new();
        for archive in &self.archives {
            refs.extend(
                registry::read_normal_shallow(&archive.record.epoch.catalog, &archive.selection)?
                    .source_authorities,
            );
        }
        refs.sort();
        refs.dedup();
        super::super::source_authority::validate_references(&refs)?;
        Ok(refs)
    }
}
pub(in crate::project) fn validate_references(refs: &[QuarantineReference]) -> StoreResult<()> {
    if refs.len() > MAX_HOLDS {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    for reference in refs {
        reference.validate()?;
    }
    if refs
        .windows(2)
        .any(|w| w[0].operation_id >= w[1].operation_id)
    {
        return Err(invalid());
    }
    Ok(())
}
pub(in crate::project) fn read(
    root: &Path,
    catalog: &RecordCatalog,
    refs: &[QuarantineReference],
    stop: &AtomicBool,
) -> StoreResult<ArchiveSet> {
    validate_references(refs)?;
    let mut archives = Vec::new();
    let mut total = 0usize;
    for reference in refs {
        checkpoint(stop)?;
        database::directory(&root.join("quarantines"))?;
        let dir = root
            .join("quarantines")
            .join(registry::instance_id(&reference.operation_id)?);
        database::directory(&dir)?;
        let record_bytes = bounded(&dir.join("record.json"), registry::MAX_REGISTRY, &mut total)?;
        let record: QuarantineRecord =
            serde_json::from_slice(&record_bytes).map_err(|_| invalid())?;
        record.validate()?;
        if record.bytes()? != record_bytes
            || QuarantineReference::from_record(&record)? != *reference
        {
            return Err(invalid());
        }
        let selection = bounded(
            &dir.join("selection.json"),
            registry::MAX_REGISTRY,
            &mut total,
        )?;
        let previous = registry::read_normal_shallow(catalog, &selection)?;
        let evidence = bounded(
            &dir.join("evidence.json"),
            super::model::MAX_EVIDENCE,
            &mut total,
        )?;
        if previous.selection != record.previous
            || previous.epoch != record.epoch
            || evidence.len() != record.evidence_length
            || digest("project-quarantine-evidence", &evidence) != record.evidence_digest
            || previous
                .retained_quarantines
                .iter()
                .any(|r| refs.binary_search(r).is_err())
        {
            return Err(invalid());
        }
        archives.push(Archive {
            reference: reference.clone(),
            record,
            selection,
            evidence,
        });
    }
    // Dependency revisions are strictly earlier. This also rejects self-reference
    // and cycles without recursively decoding any historical selection.
    for archive in &archives {
        let previous = registry::read_normal_shallow(catalog, &archive.selection)?;
        for dependency in previous.retained_quarantines {
            let other = archives
                .iter()
                .find(|a| a.reference == dependency)
                .ok_or_else(invalid)?;
            if other.record.revision >= previous.selection.revision() {
                return Err(invalid());
            }
        }
    }
    Ok(ArchiveSet {
        references: refs.to_vec(),
        archives,
    })
}
fn bounded(path: &Path, max: usize, total: &mut usize) -> StoreResult<Vec<u8>> {
    database::regular(path, max as u64)?;
    let length = usize::try_from(fs::symlink_metadata(path).map_err(|_| invalid())?.len())
        .map_err(|_| invalid())?;
    *total = total
        .checked_add(length)
        .filter(|n| *n <= MAX_BYTES)
        .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
    let bytes = registry::read_file(path, max)?;
    if bytes.len() != length {
        return Err(invalid());
    }
    Ok(bytes)
}
pub(in crate::project) fn write(
    root: &Path,
    set: &ArchiveSet,
    stop: &AtomicBool,
) -> StoreResult<()> {
    for archive in &set.archives {
        checkpoint(stop)?;
        directory(&root.join("quarantines"))?;
        let dir = archive.record.archive(root)?;
        directory(&dir)?;
        replacement::write_exact_or_new(&dir.join("selection.json"), &archive.selection)?;
        replacement::write_exact_or_new(&dir.join("record.json"), &archive.record.bytes()?)?;
        super::write_evidence(&dir.join("evidence.json"), &archive.evidence)?;
    }
    Ok(())
}
pub(in crate::project) fn directory(path: &Path) -> StoreResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => database::directory(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            database::create_private_directory(path)
                .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))
        }
        Err(_) => Err(invalid()),
    }
}
