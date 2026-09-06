use super::super::{remote::Remote, state};
use crate::{Result, git};
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Fixture {
    pub root: PathBuf,
    pub upstream: PathBuf,
    pub checkout: PathBuf,
    pub initial: String,
}
impl Fixture {
    pub fn new(format: &str, shallow: bool) -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "wdf-update-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let upstream = root.join("upstream");
        let checkout = root.join("checkout space");
        fs::create_dir(&upstream)?;
        git::isolated_text(
            &upstream,
            &[
                "init",
                "-q",
                "--initial-branch=live",
                &format!("--object-format={format}"),
            ],
        )?;
        configure(&upstream)?;
        put(&upstream, "data.txt", "initial\n")?;
        put(&upstream, "version.txt", "99.0.0.1\n")?;
        put(&upstream, ".gitignore", "ignored\n")?;
        let initial = commit(&upstream)?;
        let mut clone = vec!["clone", "--no-local", "--no-tags"];
        if shallow {
            clone.push("--depth=1");
        }
        clone.extend([
            upstream.to_str().ok_or("fixture path")?,
            checkout.to_str().ok_or("fixture path")?,
        ]);
        git::isolated_text(&root, &clone)?;
        configure(&checkout)?;
        git::isolated_text(
            &checkout,
            &[
                "remote",
                "set-url",
                "origin",
                "https://example.invalid/source.git",
            ],
        )?;
        Ok(Self {
            root,
            upstream,
            checkout,
            initial,
        })
    }
    pub fn advance(&self, path: &str, value: &str) -> Result<String> {
        put(&self.upstream, path, value)?;
        commit(&self.upstream)
    }
    pub fn head(&self) -> Result<String> {
        state::resolve(&self.checkout, "HEAD")
    }
    pub fn remote(&self) -> Local<'_> {
        Local {
            path: &self.upstream,
            heads: Cell::new(0),
            fetches: Cell::new(0),
        }
    }
    pub fn lock_exists(&self) -> bool {
        self.checkout.join(".git/wow-source-update.lock").exists()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn configure(root: &Path) -> Result<()> {
    for (key, value) in [
        ("user.name", "Synthetic update test"),
        ("user.email", "test@example.invalid"),
        ("core.autocrlf", "false"),
    ] {
        git::isolated_text(root, &["config", key, value])?;
    }
    Ok(())
}
pub fn put(root: &Path, path: &str, value: &str) -> Result<()> {
    let target = root.join(path);
    fs::create_dir_all(target.parent().ok_or("fixture parent")?)?;
    fs::write(target, value)?;
    Ok(())
}
pub fn commit(root: &Path) -> Result<String> {
    git::isolated_text(root, &["add", "."])?;
    git::isolated_text(
        root,
        &[
            "-c",
            "commit.gpgSign=false",
            "commit",
            "-qm",
            "synthetic source",
        ],
    )?;
    state::resolve(root, "HEAD")
}
pub struct Local<'a> {
    pub path: &'a Path,
    pub heads: Cell<usize>,
    pub fetches: Cell<usize>,
}
impl Remote for Local<'_> {
    fn head(&self, _: &Path, _: &str, branch: &str) -> Result<String> {
        self.heads.set(self.heads.get() + 1);
        state::resolve(self.path, &format!("refs/heads/{branch}"))
    }
    fn fetch(&self, root: &Path, _: &str, revision: &str) -> Result<()> {
        self.fetches.set(self.fetches.get() + 1);
        git::isolated_text(
            root,
            &[
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                "--no-recurse-submodules",
                "--no-auto-maintenance",
                "--refmap=",
                self.path.to_str().ok_or("fixture path")?,
                revision,
            ],
        )?;
        Ok(())
    }
}
