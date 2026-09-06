//! Cooperative lock and minimal interrupted-update journal, never a store port.
use crate::Result;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(super) struct Lock {
    path: PathBuf,
    file: Option<File>,
    retain: bool,
    released: bool,
}
impl Lock {
    pub fn acquire(root: &Path, expected: &str) -> Result<Self> {
        let path = root.join(".git/wow-source-update.lock");
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
            writeln!(
                file,
                "schema=wow-source-update-lock/1\nexpected_head={expected}"
            )?;
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
        fs::remove_file(&self.path)?;
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
