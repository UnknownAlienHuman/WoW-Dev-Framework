//! Cooperative lock and minimal interrupted-update journal, never a store port.
use crate::Result;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const LOCK_SCHEMA: &str = "wow-source-update-lock/1";
const MAX_LOCK_BYTES: u64 = 4096;

pub(super) struct Lock {
    path: PathBuf,
    file: Option<File>,
    retain: bool,
    released: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LockRecord {
    pub expected_head: String,
    pub selected_revision: String,
}

impl Lock {
    pub fn acquire(root: &Path, expected: &str) -> Result<Self> {
        let path = lock_path(root);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| "source update lock exists or is unavailable; inspect before retrying")?;
        let mut lock = Self {
            path,
            file: Some(file),
            retain: false,
            released: false,
        };
        if let Some(file) = lock.file.as_mut() {
            writeln!(file, "schema={LOCK_SCHEMA}\nexpected_head={expected}")?;
            file.sync_all()?;
        }
        Ok(lock)
    }

    pub fn prepare(&mut self, selected: &str) -> Result<()> {
        let file = self.file.as_mut().ok_or("source update lock is closed")?;
        writeln!(file, "selected_revision={selected}\nphase=applying")?;
        file.sync_all()?;
        self.retain = true;
        Ok(())
    }

    pub fn release(&mut self) -> Result<()> {
        drop(self.file.take());
        remove_exact(&self.path)?;
        self.released = true;
        Ok(())
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.retain && !self.released {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) fn read(root: &Path) -> Result<Option<LockRecord>> {
    let path = lock_path(root);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_LOCK_BYTES {
        return Err("source update lock is not a safe bounded regular file".into());
    }
    let mut bytes = Vec::new();
    File::open(&path)?
        .take(MAX_LOCK_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > MAX_LOCK_BYTES {
        return Err("source update lock changed while being read".into());
    }
    let text = std::str::from_utf8(&bytes)?;
    let mut schema = None;
    let mut expected_head = None;
    let mut selected_revision = None;
    let mut phase = None;
    for line in text.lines().filter(|line| !line.is_empty()) {
        let (key, value) = line
            .split_once('=')
            .ok_or("source update lock record is malformed")?;
        let target = match key {
            "schema" => &mut schema,
            "expected_head" => &mut expected_head,
            "selected_revision" => &mut selected_revision,
            "phase" => &mut phase,
            _ => return Err("source update lock contains an unknown field".into()),
        };
        if target.replace(value).is_some() {
            return Err("source update lock contains a duplicate field".into());
        }
    }
    if schema != Some(LOCK_SCHEMA) || phase != Some("applying") {
        return Err("source update lock has an unsupported schema or phase".into());
    }
    let expected_head = expected_head.ok_or("source update lock is missing expected HEAD")?;
    let selected_revision =
        selected_revision.ok_or("source update lock is missing selected revision")?;
    if !crate::git::oid(expected_head)
        || !crate::git::oid(selected_revision)
        || expected_head.len() != selected_revision.len()
    {
        return Err("source update lock contains an invalid revision".into());
    }
    Ok(Some(LockRecord {
        expected_head: expected_head.to_owned(),
        selected_revision: selected_revision.to_owned(),
    }))
}

pub(super) fn release_reconciled(
    root: &Path,
    expected_head: &str,
    selected_revision: &str,
) -> Result<()> {
    let record = read(root)?.ok_or("source update reconciliation lock is absent")?;
    if record.expected_head != expected_head || record.selected_revision != selected_revision {
        return Err("source update reconciliation lock belongs to another operation".into());
    }
    remove_exact(&lock_path(root))
}

fn lock_path(root: &Path) -> PathBuf {
    root.join(".git/wow-source-update.lock")
}

fn remove_exact(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("source update lock is not a regular file".into());
    }
    fs::remove_file(path)?;
    Ok(())
}
