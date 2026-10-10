//! Flat portable original-epoch evidence, independently admitted from selected SQL.
mod io;
mod model;
use super::{
    model::{checkpoint, failure, invalid},
    quarantine::archives::{self, ArchiveSet, QuarantineReference},
    registry::{self, AdmittedRegistry},
};
use crate::{StoreErrorCode, StoreResult};
pub use model::SourceAuthorityReference;
use model::{MAX_AUTHORITIES, MAX_BYTES, SourceAuthorityManifest};
use std::{
    collections::{BTreeMap, btree_map::Entry},
    path::Path,
    sync::atomic::AtomicBool,
};

struct AuthorityEntry {
    reference: SourceAuthorityReference,
    manifest: SourceAuthorityManifest,
    selection: Vec<u8>,
    archives: ArchiveSet,
}
impl super::ProjectStore {
    /// Observe fully admitted original-epoch evidence retained by this selector.
    pub fn retained_source_authorities(&self) -> StoreResult<Vec<SourceAuthorityReference>> {
        self.db.ensure_idle()?;
        Ok(registry::read(&self.db.root, &self.db.epoch.catalog)?.source_authorities)
    }
}
pub(in crate::project) struct AuthoritySet {
    references: Vec<SourceAuthorityReference>,
    entries: Vec<AuthorityEntry>,
}
impl AuthoritySet {
    pub fn empty() -> Self {
        Self {
            references: Vec::new(),
            entries: Vec::new(),
        }
    }
    pub fn references(&self) -> &[SourceAuthorityReference] {
        &self.references
    }
    pub fn byte_length(&self) -> StoreResult<usize> {
        self.entries.iter().try_fold(0usize, |total, entry| {
            let archive_length = entry.archives.byte_length()?;
            total
                .checked_add(entry.manifest.bytes()?.len())
                .and_then(|n| n.checked_add(entry.selection.len()))
                .and_then(|n| n.checked_add(archive_length))
                .filter(|n| *n <= MAX_BYTES)
                .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))
        })
    }
    fn hold_inventory(
        &self,
    ) -> StoreResult<BTreeMap<(super::EpochId, crate::OperationId), QuarantineReference>> {
        let mut inventory = BTreeMap::new();
        for entry in &self.entries {
            insert_holds(&mut inventory, &entry.archives)?;
        }
        Ok(inventory)
    }
    pub fn hold_count(&self) -> StoreResult<usize> {
        Ok(self.hold_inventory()?.len())
    }
    pub fn validate_closure(&self) -> StoreResult<()> {
        validate_references(&self.references)?;
        if self.entries.len() != self.references.len() || self.entries.len() > MAX_AUTHORITIES {
            return Err(invalid());
        }
        for (entry, reference) in self.entries.iter().zip(&self.references) {
            if &entry.reference != reference
                || SourceAuthorityReference::from_bytes(&entry.manifest.bytes()?)? != *reference
                || entry.selection.len() != entry.manifest.selection_length
                || super::model::digest("project-registry", &entry.selection)
                    != entry.manifest.selection.digest()
                || entry.archives.references() != entry.manifest.retained_quarantines
            {
                return Err(invalid());
            }
            entry.archives.validate_epoch(&entry.manifest.epoch)?;
            let preceding =
                registry::read_normal_shallow(&entry.manifest.epoch.catalog, &entry.selection)?;
            if preceding.epoch != entry.manifest.epoch
                || preceding.selection != entry.manifest.selection
                || preceding.retained_quarantines != entry.manifest.retained_quarantines
                || (!entry.archives.references().is_empty()
                    && entry.archives.max_revision() >= preceding.selection.revision())
            {
                return Err(invalid());
            }
            let mut dependencies = preceding.source_authorities;
            dependencies.extend(entry.archives.required_authorities()?);
            dependencies.sort();
            dependencies.dedup();
            validate_references(&dependencies)?;
            if dependencies != entry.manifest.dependencies
                || dependencies
                    .iter()
                    .any(|dependency| self.references.binary_search(dependency).is_err())
            {
                return Err(invalid());
            }
        }
        // Admit a finite flat DAG without following historical filesystem paths.
        let mut admitted = vec![false; self.entries.len()];
        loop {
            let mut changed = false;
            for (index, entry) in self.entries.iter().enumerate() {
                if !admitted[index]
                    && entry.manifest.dependencies.iter().all(|dependency| {
                        self.references
                            .binary_search(dependency)
                            .ok()
                            .is_some_and(|other| admitted[other])
                    })
                {
                    admitted[index] = true;
                    changed = true;
                }
            }
            if admitted.iter().all(|done| *done) {
                break;
            }
            if !changed {
                return Err(invalid());
            }
        }
        self.byte_length()?;
        self.hold_count()?;
        Ok(())
    }
    /// Account selected and archived authority together before any effects.
    pub fn admit_selected(&self, selected: &ArchiveSet) -> StoreResult<()> {
        self.validate_closure()?;
        if selected
            .required_authorities()?
            .iter()
            .any(|r| self.references.binary_search(r).is_err())
        {
            return Err(invalid());
        }
        let mut inventory = self.hold_inventory()?;
        insert_holds(&mut inventory, selected)?;
        self.byte_length()?
            .checked_add(selected.byte_length()?)
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
        Ok(())
    }
    pub fn admit_hold(
        &self,
        selected: &ArchiveSet,
        record: &super::quarantine::model::QuarantineRecord,
        selection_bytes: usize,
        evidence_bytes: usize,
    ) -> StoreResult<()> {
        self.admit_selected(selected)?;
        let mut inventory = self.hold_inventory()?;
        insert_holds(&mut inventory, selected)?;
        let key = (record.epoch.epoch_id().clone(), record.operation_id.clone());
        if inventory.contains_key(&key) {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        if inventory.len() >= archives::MAX_HOLDS {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        let record_length = record.bytes()?.len();
        self.byte_length()?
            .checked_add(selected.byte_length()?)
            .and_then(|n| n.checked_add(selection_bytes))
            .and_then(|n| n.checked_add(evidence_bytes))
            .and_then(|n| n.checked_add(record_length))
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
        Ok(())
    }
    pub fn merge(self, other: Self) -> StoreResult<Self> {
        self.validate_closure()?;
        other.validate_closure()?;
        let mut entries = BTreeMap::new();
        for mut entry in self.entries.into_iter().chain(other.entries) {
            let key = entry.reference.manifest_digest().to_owned();
            match entries.entry(key) {
                Entry::Vacant(place) => {
                    place.insert(entry);
                }
                Entry::Occupied(place) => {
                    let previous: AuthorityEntry = place.remove();
                    if previous.reference != entry.reference
                        || previous.manifest != entry.manifest
                        || previous.selection != entry.selection
                    {
                        return Err(failure(StoreErrorCode::OperationConflict));
                    }
                    entry.archives = previous.archives.merge(entry.archives)?;
                    entries.insert(entry.reference.manifest_digest().to_owned(), entry);
                }
            }
        }
        let entries: Vec<_> = entries.into_values().collect();
        let result = Self {
            references: entries.iter().map(|e| e.reference.clone()).collect(),
            entries,
        };
        result.validate_closure()?;
        Ok(result)
    }
}
fn insert_holds(
    inventory: &mut BTreeMap<(super::EpochId, crate::OperationId), QuarantineReference>,
    archives: &ArchiveSet,
) -> StoreResult<()> {
    for (epoch, reference) in archives.hold_identities() {
        let key = (epoch, reference.operation_id().clone());
        if inventory
            .get(&key)
            .is_some_and(|previous| previous != &reference)
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        inventory.insert(key, reference);
        if inventory.len() > archives::MAX_HOLDS {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
    }
    Ok(())
}
pub(in crate::project) fn validate_references(
    refs: &[SourceAuthorityReference],
) -> StoreResult<()> {
    model::validate_references(refs)
}
pub(in crate::project) fn read(
    root: &Path,
    refs: &[SourceAuthorityReference],
    stop: &AtomicBool,
) -> StoreResult<AuthoritySet> {
    io::read(root, refs, stop)
}
pub(in crate::project) fn write(
    root: &Path,
    set: &AuthoritySet,
    stop: &AtomicBool,
) -> StoreResult<()> {
    io::write(root, set, stop)
}
/// Admit the complete future union before creating a replacement instance.
/// Activation repeats admission against freshly observed native evidence.
pub(in crate::project) fn preflight_transport(
    root: &Path,
    selected: &AdmittedRegistry,
    target: &super::VerifiedBackup,
    stop: &AtomicBool,
) -> StoreResult<()> {
    if &selected.epoch != target.manifest().epoch() {
        return Err(invalid());
    }
    let held = archives::read(
        root,
        &selected.epoch.catalog,
        &selected.retained_quarantines,
        stop,
    )?;
    held.validate_epoch(&selected.epoch)?;
    let incoming = archives::read(
        &target.root,
        &selected.epoch.catalog,
        target.manifest().retained_quarantines(),
        stop,
    )?;
    incoming.validate_epoch(&selected.epoch)?;
    let closure = held.merge(incoming)?;
    let sources = read(root, &selected.source_authorities, stop)?.merge(read(
        &target.root,
        target.manifest().source_authorities(),
        stop,
    )?)?;
    sources.admit_selected(&closure)
}
/// Capture exact original selector/holds only after the caller's native live-source guard.
pub(in crate::project) fn capture_source(
    root: &Path,
    admitted: &AdmittedRegistry,
    snapshot: &str,
    stop: &AtomicBool,
) -> StoreResult<AuthoritySet> {
    checkpoint(stop)?;
    if admitted.quarantine.is_some() {
        return Err(failure(StoreErrorCode::Quarantined));
    }
    let inherited = read(root, &admitted.source_authorities, stop)?;
    let selection =
        registry::read_file(&root.join(registry::REGISTRY_FILE), registry::MAX_REGISTRY)?;
    let actual = registry::read_normal_shallow(&admitted.epoch.catalog, &selection)?;
    if actual.selection != admitted.selection
        || actual.epoch != admitted.epoch
        || actual.retained_quarantines != admitted.retained_quarantines
        || actual.source_authorities != admitted.source_authorities
    {
        return Err(failure(StoreErrorCode::CurrentConflict));
    }
    let archives = archives::read(
        root,
        &admitted.epoch.catalog,
        &admitted.retained_quarantines,
        stop,
    )?;
    archives.validate_epoch(&admitted.epoch)?;
    inherited.admit_selected(&archives)?;
    let mut dependencies = admitted.source_authorities.clone();
    dependencies.extend(archives.required_authorities()?);
    dependencies.sort();
    dependencies.dedup();
    let manifest = SourceAuthorityManifest {
        schema: "wow-store/project-source-authority/1".into(),
        epoch: admitted.epoch.clone(),
        selection: admitted.selection.clone(),
        selection_length: selection.len(),
        source_snapshot: snapshot.to_owned(),
        retained_quarantines: admitted.retained_quarantines.clone(),
        dependencies,
    };
    let reference = SourceAuthorityReference::from_bytes(&manifest.bytes()?)?;
    let mut entries = inherited.entries;
    entries.push(AuthorityEntry {
        reference,
        manifest,
        selection,
        archives,
    });
    entries.sort_by(|a, b| a.reference.cmp(&b.reference));
    let result = AuthoritySet {
        references: entries.iter().map(|e| e.reference.clone()).collect(),
        entries,
    };
    result.validate_closure()?;
    checkpoint(stop)?;
    Ok(result)
}
