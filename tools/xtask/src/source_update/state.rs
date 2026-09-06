//! Local guards. This requires an exclusively owned, trusted standalone checkout.
use crate::{Result, git, manifest};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct State {
    pub head: String,
    pub branch: String,
    pub origin: String,
    pub config_hash: String,
}

pub(super) fn root(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize()?;
    let metadata = fs::symlink_metadata(path.join(".git"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("source update requires a standalone non-bare checkout".into());
    }
    let top = git::isolated_text(&path, &["rev-parse", "--show-toplevel"])?;
    if Path::new(&top).canonicalize()? != path {
        return Err("source root differs from the Git worktree".into());
    }
    Ok(path)
}

pub(super) fn inspect(root: &Path) -> Result<State> {
    for path in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "BISECT_LOG",
        "rebase-merge",
        "rebase-apply",
        "index.lock",
        "info/grafts",
    ] {
        match fs::symlink_metadata(root.join(".git").join(path)) {
            Ok(_) => {
                return Err(
                    "source checkout has an unfinished operation or overridden ancestry".into(),
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let metadata = fs::symlink_metadata(root.join(".git/config"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("source config is not a regular file".into());
    }
    let config = git::isolated_run(
        root,
        &["config", "--local", "--no-includes", "--null", "--list"],
        None,
        1024 * 1024,
    )?;
    for record in config.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let key = record
            .split(|b| *b == b'\n')
            .next()
            .ok_or("invalid Git config")?;
        let key = std::str::from_utf8(key)?.to_ascii_lowercase();
        if key.starts_with("filter.")
            || key.starts_with("include.")
            || key.starts_with("includeif.")
            || key.starts_with("url.")
            || key.starts_with("credential.")
            || key == "core.sparsecheckout"
            || key == "core.sparsecheckoutcone"
            || key == "core.askpass"
            || (key.starts_with("branch.") && key.ends_with(".mergeoptions"))
            || key == "extensions.worktreeconfig"
            || key == "core.attributesfile"
            || (key.starts_with("http.") && key.ends_with("extraheader"))
        {
            return Err(
                "source update refuses filters, includes, rewrites, credentials or sparse config"
                    .into(),
            );
        }
    }
    // These index flags can hide edited worktree bytes from ordinary status.
    let index = git::isolated_run(root, &["ls-files", "-v", "-z"], None, 16 * 1024 * 1024)?;
    if index
        .split(|b| *b == 0)
        .filter(|e| !e.is_empty())
        .any(|e| e[0].is_ascii_lowercase() || e[0] == b'S')
    {
        return Err("source index contains assume-unchanged or skip-worktree entries".into());
    }
    let status = git::isolated_run(
        root,
        &[
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--ignore-submodules=none",
            "-z",
        ],
        None,
        16 * 1024 * 1024,
    )?;
    if !status.is_empty() {
        return Err("source checkout has local changes; checkout update refused".into());
    }
    let branch = git::isolated_text(root, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .map_err(|_| "source checkout is detached; no implicit branch selection")?;
    let origin = git::isolated_text(root, &["remote", "get-url", "origin"])?;
    super::remote::validate_origin(&origin)?;
    Ok(State {
        head: resolve(root, "HEAD")?,
        branch,
        origin,
        config_hash: manifest::digest(&config),
    })
}

pub(super) fn resolve(root: &Path, selector: &str) -> Result<String> {
    let revision = git::isolated_text(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{selector}^{{commit}}"),
        ],
    )?;
    if !git::oid(&revision) {
        return Err("noncanonical source commit".into());
    }
    Ok(revision)
}
