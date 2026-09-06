//! Fixed local Git reads and new-only output for development drivers.
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

pub(super) fn validate_path(path: &str) -> Result<(), &'static str> {
    if path.is_empty()
        || path.len() > 4096
        || path.contains([':', '\\'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
    {
        return Err("unsafe source-relative path");
    }
    Ok(())
}
pub(super) fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
pub(super) fn git(
    root: &Path,
    args: &[&str],
    limit: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut child = Command::new("git")
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "core.hooksPath=",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "submodule.recurse=false",
        ])
        .args(args)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_NAMESPACE")
        .env_remove("GIT_CONFIG_COUNT")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut bytes = Vec::new();
    let result = child
        .stdout
        .take()
        .ok_or("missing git output")?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes);
    if result.is_err() || bytes.len() > limit {
        let _ = child.kill();
        let _ = child.wait();
        result?;
        return Err("Git object exceeds input limit".into());
    }
    if !child.wait()?.success() {
        return Err(
            "Git could not read the selected local object; materialize missing objects explicitly"
                .into(),
        );
    }
    Ok(bytes)
}
