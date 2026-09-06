//! Reviewed test-only consumer adapters, not the production semantic analyzer.
pub mod fixture;
pub mod mutations;
pub mod package;
mod paths;
pub mod process;
pub mod report;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Workspace {
    pub path: PathBuf,
    retain: bool,
}
impl Workspace {
    pub fn new() -> Result<Self> {
        let explicit = std::env::var_os("WDF_CONSUMER_OUTPUT");
        let retain = explicit.is_some();
        let path = explicit.map(PathBuf::from).unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "wdf-consumers-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ))
        });
        // New-only, never repurpose the operator's existing directory.
        fs::create_dir(&path)?;
        let path = paths::canonical_root(&path)?;
        fs::create_dir(path.join("home"))?;
        Ok(Self { path, retain })
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        if !self.retain {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
