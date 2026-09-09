//! Explicit multi-file annotation resources; moving refs resolve once per input.
use super::io::{git, validate_path};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    ffi::OsStr,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_annotations::aliases::{MAX_CATALOG_ALIASES, MAX_CATALOG_BYTES, MAX_CATALOG_FILES};
use wow_reference::native::source_digest;
use wow_reference::native_aliases::{AliasDocument, ingest_alias_catalog};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn read(
    inputs: &[(&OsStr, &OsStr, &OsStr)],
    cancelled: &AtomicBool,
) -> Result<Vec<AliasDocument>> {
    if inputs.len() > MAX_CATALOG_FILES {
        return Err("alias catalog file limit".into());
    }
    let mut resolutions = BTreeMap::new();
    let mut generation = None;
    let mut paths = BTreeSet::new();
    let mut selected = Vec::new();
    // Freeze all identities before reading resources. Equivalent checkout paths
    // share a resolution, so repeated HEAD arguments cannot select moving worlds.
    for (checkout, selector, path) in inputs {
        if cancelled.load(Ordering::Relaxed) {
            return Err("alias acquisition cancelled".into());
        }
        let root = PathBuf::from(*checkout);
        let selector = selector.to_str().ok_or("alias ref is not UTF-8")?;
        let path = path.to_str().ok_or("alias path is not UTF-8")?;
        validate_path(path)?;
        if selector.is_empty()
            || selector.starts_with('-')
            || selector.chars().any(char::is_control)
        {
            return Err("invalid alias source ref".into());
        }
        if !paths.insert(path) {
            return Err("duplicate alias resource path".into());
        }
        let key = (root.canonicalize()?, selector.to_owned());
        if let Entry::Vacant(entry) = resolutions.entry(key.clone()) {
            let resolved = git(
                &root,
                &[
                    "rev-parse",
                    "--verify",
                    "--end-of-options",
                    &format!("{selector}^{{commit}}"),
                ],
                128,
            )?;
            let revision = std::str::from_utf8(&resolved)?.trim();
            if !matches!(revision.len(), 40 | 64)
                || !revision.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("invalid resolved alias revision".into());
            }
            entry.insert(revision.to_owned());
        }
        let revision = resolutions.get(&key).ok_or("missing alias resolution")?;
        if generation
            .as_ref()
            .is_some_and(|expected| expected != revision)
        {
            return Err("alias resources must share one exact revision".into());
        }
        generation = Some(revision.clone());
        selected.push((root, revision.clone(), path));
    }
    selected.sort_by(|a, b| a.2.cmp(b.2));
    let mut documents = Vec::new();
    let mut total_bytes = 0usize;
    let mut total_aliases = 0usize;
    for (root, revision, path) in selected {
        if cancelled.load(Ordering::Relaxed) {
            return Err("alias acquisition cancelled".into());
        }
        let entry = git(&root, &["ls-tree", "-z", &revision, "--", path], 8192)?;
        let entry = std::str::from_utf8(&entry)?;
        if !(entry.starts_with("100644 blob ") || entry.starts_with("100755 blob "))
            || !entry.ends_with(&format!("\t{path}\0"))
            || entry.matches('\0').count() != 1
        {
            return Err("alias resource must be an exact regular Git blob".into());
        }
        let bytes = git(
            &root,
            &["cat-file", "blob", &format!("{revision}:{path}")],
            256 * 1024,
        )?;
        total_bytes += bytes.len();
        if total_bytes > MAX_CATALOG_BYTES {
            return Err("alias catalog aggregate byte limit".into());
        }
        let document = ingest_alias_catalog(
            &revision,
            path,
            std::str::from_utf8(&bytes)?,
            &source_digest(&bytes),
            cancelled,
        )?;
        total_aliases += document.aliases().len()
            + document.structures().len()
            + document.namespaces().len()
            + document
                .function_containers()
                .iter()
                .map(|container| 1 + container.methods.len())
                .sum::<usize>();
        if total_aliases > MAX_CATALOG_ALIASES {
            return Err("alias catalog aggregate declaration limit".into());
        }
        documents.push(document);
    }
    Ok(documents)
}
