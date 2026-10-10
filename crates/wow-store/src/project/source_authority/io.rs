//! Confined, bounded transport of canonical original-source authority.

use super::{
    AuthorityEntry, AuthoritySet,
    model::{
        MAX_BYTES, MAX_MANIFEST, SourceAuthorityManifest, SourceAuthorityReference,
        validate_references,
    },
};
use crate::project::{
    database,
    model::{checkpoint, encode, failure, invalid},
    quarantine::archives,
    registry, replacement,
};
use crate::{StoreErrorCode, StoreResult};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub(super) fn read(
    root: &Path,
    refs: &[SourceAuthorityReference],
    stop: &AtomicBool,
) -> StoreResult<AuthoritySet> {
    checkpoint(stop)?;
    validate_references(refs)?;
    database::directory(root)?;
    if !refs.is_empty() {
        database::directory(&root.join("source-authorities"))?;
    }

    let mut entries = Vec::new();
    let mut total = 0usize;
    for reference in refs {
        checkpoint(stop)?;
        let directory = entry_directory(root, reference)?;
        database::directory(&directory)?;
        let bytes = bounded(&directory.join("authority.json"), MAX_MANIFEST, &mut total)?;
        if SourceAuthorityReference::from_bytes(&bytes)? != *reference {
            return Err(invalid());
        }
        let manifest: SourceAuthorityManifest =
            serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if manifest.bytes()? != bytes {
            return Err(invalid());
        }
        let selection = bounded(
            &directory.join("selection.json"),
            registry::MAX_REGISTRY,
            &mut total,
        )?;
        let archives = archives::read(
            &directory,
            &manifest.epoch.catalog,
            &manifest.retained_quarantines,
            stop,
        )?;
        let entry = AuthorityEntry {
            reference: reference.clone(),
            manifest,
            selection,
            archives,
        };
        validate_entry(&entry)?;
        charge(&mut total, entry.archives.byte_length()?)?;
        entries.push(entry);
    }

    let set = AuthoritySet {
        references: refs.to_vec(),
        entries,
    };
    set.validate_closure()?;
    checkpoint(stop)?;
    Ok(set)
}

pub(super) fn write(root: &Path, set: &AuthoritySet, stop: &AtomicBool) -> StoreResult<()> {
    checkpoint(stop)?;
    set.validate_closure()?;
    let mut total = 0usize;
    charge(&mut total, set.byte_length()?)?;
    for entry in &set.entries {
        checkpoint(stop)?;
        validate_entry(entry)?;
    }
    database::directory(root)?;
    if !set.entries.is_empty() {
        archives::directory(&root.join("source-authorities"))?;
    }
    for entry in &set.entries {
        checkpoint(stop)?;
        let directory = entry_directory(root, &entry.reference)?;
        archives::directory(&directory)?;
        replacement::write_exact_or_new(&directory.join("selection.json"), &entry.selection)?;
        archives::write(&directory, &entry.archives, stop)?;
        checkpoint(stop)?;
        replacement::write_exact_or_new(
            &directory.join("authority.json"),
            &entry.manifest.bytes()?,
        )?;
    }
    checkpoint(stop)
}

fn validate_entry(entry: &AuthorityEntry) -> StoreResult<()> {
    let manifest = &entry.manifest;
    if SourceAuthorityReference::from_bytes(&manifest.bytes()?)? != entry.reference
        || entry.selection.len() != manifest.selection_length
        || entry.selection.len() > registry::MAX_REGISTRY
        || entry.archives.references() != manifest.retained_quarantines.as_slice()
    {
        return Err(invalid());
    }
    database::admit_epoch(&encode(&manifest.epoch, 65536)?, &manifest.epoch.catalog)?;
    let selected = registry::read_normal_shallow(&manifest.epoch.catalog, &entry.selection)?;
    if selected.selection != manifest.selection
        || selected.epoch != manifest.epoch
        || selected.retained_quarantines != manifest.retained_quarantines
    {
        return Err(invalid());
    }
    entry.archives.validate_epoch(&manifest.epoch)?;
    let mut dependencies = selected.source_authorities;
    dependencies.extend(entry.archives.required_authorities()?);
    dependencies.sort_unstable();
    dependencies.dedup();
    validate_references(&dependencies)?;
    if dependencies != manifest.dependencies {
        return Err(invalid());
    }
    Ok(())
}

fn entry_directory(root: &Path, reference: &SourceAuthorityReference) -> StoreResult<PathBuf> {
    reference.validate()?;
    let hash = reference
        .manifest_digest
        .strip_prefix("project-source-authority:sha256:")
        .ok_or_else(invalid)?;
    Ok(root.join("source-authorities").join(hash))
}

fn bounded(path: &Path, max: usize, total: &mut usize) -> StoreResult<Vec<u8>> {
    database::regular(path, max as u64)?;
    let length = usize::try_from(fs::symlink_metadata(path).map_err(|_| invalid())?.len())
        .map_err(|_| invalid())?;
    charge(total, length)?;
    let bytes = registry::read_file(path, max)?;
    if bytes.len() != length {
        return Err(invalid());
    }
    Ok(bytes)
}

fn charge(total: &mut usize, bytes: usize) -> StoreResult<()> {
    *total = total
        .checked_add(bytes)
        .filter(|length| *length <= MAX_BYTES)
        .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
    Ok(())
}
