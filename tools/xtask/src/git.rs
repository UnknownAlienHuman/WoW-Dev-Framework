use crate::Result;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Fixed Git subprocesses: no shell, no inherited Git root overrides, bounded
/// output and deadline. Stderr is discarded: it can contain credentialed URLs.
pub fn run(root: &Path, args: &[&str], input: Option<Vec<u8>>, limit: usize) -> Result<Vec<u8>> {
    execute(root, args, input, limit, false)
}

/// Source-update commands do not inherit global/system configuration or attributes.
pub fn isolated_run(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    limit: usize,
) -> Result<Vec<u8>> {
    execute(root, args, input, limit, true)
}
pub fn isolated_text(root: &Path, args: &[&str]) -> Result<String> {
    Ok(
        String::from_utf8(isolated_run(root, args, None, 4 * 1024 * 1024)?)?
            .trim()
            .to_owned(),
    )
}
/// Admit source configuration before any command can inspect worktree contents.
/// Shared by advisory freshness and the existing managed-update prerequisite.
pub(crate) fn admit_source_config(root: &Path) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(root.join(".git/config"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("source config is not a regular file".into());
    }
    let config = isolated_run(
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
                "source admission refuses filters, includes, rewrites, credentials or sparse config"
                    .into(),
            );
        }
    }
    let index = isolated_run(root, &["ls-files", "-v", "-z"], None, 16 * 1024 * 1024)?;
    if index
        .split(|b| *b == 0)
        .filter(|e| !e.is_empty())
        .any(|e| e[0].is_ascii_lowercase() || e[0] == b'S')
    {
        return Err("source index contains assume-unchanged or skip-worktree entries".into());
    }
    // A recursive submodule status would enter another checkout's configuration.
    let entries = isolated_run(root, &["ls-files", "--stage", "-z"], None, 16 * 1024 * 1024)?;
    if entries
        .split(|b| *b == 0)
        .any(|entry| entry.starts_with(b"160000 "))
    {
        return Err("source admission refuses submodule worktrees".into());
    }
    Ok(config)
}
fn execute(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    limit: usize,
    isolated: bool,
) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    if isolated {
        command
            .env_remove("SSH_ASKPASS")
            .env("GIT_ASKPASS", "")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_ATTR_NOSYSTEM", "1")
            .arg("-c")
            .arg(if cfg!(windows) {
                "core.attributesFile=NUL"
            } else {
                "core.attributesFile=/dev/null"
            });
    }
    command
        .args([
            "--no-replace-objects",
            "--literal-pathspecs",
            "-c",
            "core.fsmonitor=false",
            "-c",
        ])
        .arg(if cfg!(windows) {
            "core.hooksPath=NUL"
        } else {
            "core.hooksPath=/dev/null"
        })
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn()?;
    let reader = child.stdout.take().ok_or("missing Git output")?;
    let writer = child.stdin.take();
    let (sender, receiver) = mpsc::channel();
    let read_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = reader
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let write_thread = std::thread::spawn(move || -> std::io::Result<()> {
        if let (Some(mut writer), Some(input)) = (writer, input) {
            writer.write_all(&input)?;
        }
        Ok(())
    });
    let result = receiver.recv_timeout(Duration::from_secs(120));
    if !matches!(&result, Ok(Ok(bytes)) if bytes.len() <= limit) {
        let _ = child.kill();
        let _ = child.wait();
        let _ = read_thread.join();
        let _ = write_thread.join();
        return Err("Git failed, exceeded its output bound or deadline".into());
    }
    // stdout may close before the process exits; also bound the exit wait.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = read_thread.join();
            let _ = write_thread.join();
            return Err("Git exit deadline exceeded".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    read_thread.join().map_err(|_| "Git reader failed")?;
    write_thread.join().map_err(|_| "Git writer failed")??;
    if !status.success() {
        return Err("Git command failed; local objects or remote may be unavailable".into());
    }
    Ok(result.map_err(|_| "Git read deadline exceeded")??)
}
pub fn text(root: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(run(root, args, None, 4 * 1024 * 1024)?)?
        .trim()
        .to_owned())
}
pub fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn resolve(root: &Path, selector: &str) -> Result<String> {
    if selector.is_empty() || selector.starts_with('-') || selector.chars().any(char::is_control) {
        return Err("invalid source selector".into());
    }
    let revision = text(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{selector}^{{commit}}"),
        ],
    )?;
    if !oid(&revision) {
        return Err("noncanonical commit identifier".into());
    }
    Ok(revision)
}
/// Advisory freshness check only: never fetch, reset, stash or change branches.
/// Explicit local roots and public HTTPS remotes only; no provider discovery.
pub fn check_source(root: &Path, branch: &str) -> Result<u8> {
    let LocalSourceState {
        revision,
        dirty,
        current_branch,
        origin,
    } = inspect_source_local(root, branch)?;
    let reference = format!("refs/heads/{branch}");
    let remote = isolated_text(
        root,
        &[
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
            "-c",
            "protocol.https.allow=always",
            "ls-remote",
            "--exit-code",
            "--refs",
            &origin,
            &reference,
        ],
    );
    let remote = match remote {
        Ok(text) => parse_remote(&text, &reference)?,
        Err(_) => {
            println!(
                "{}",
                serde_json::json!({"status":"unverified_current","local_revision":revision,"dirty":dirty,"next_action":"network check unavailable; do not claim current source"})
            );
            return Ok(4);
        }
    };
    let status = if revision == remote {
        "current"
    } else {
        "different"
    };
    println!(
        "{}",
        serde_json::json!({"status":status,"local_revision":revision,"remote_revision":remote,"branch":current_branch,"requested_branch":branch,"dirty":dirty,"next_action":if status=="current" {"none"} else {"review local state, fetch the configured branch and offer a fast-forward update; do not reset"}})
    );
    Ok(if status == "current" { 0 } else { 3 })
}

pub(crate) struct LocalSourceState {
    pub revision: String,
    pub dirty: bool,
    current_branch: Option<String>,
    origin: String,
}

/// The exact local freshness phase, separately observable without remote I/O.
pub(crate) fn inspect_source_local(root: &Path, branch: &str) -> Result<LocalSourceState> {
    admit_source_config(root)?;
    isolated_text(root, &["check-ref-format", &format!("refs/heads/{branch}")])?;
    let revision = isolated_text(root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    if !oid(&revision) {
        return Err("noncanonical commit identifier".into());
    }
    let current_branch = isolated_text(root, &["symbolic-ref", "--quiet", "--short", "HEAD"]).ok();
    let origin = isolated_text(root, &["remote", "get-url", "origin"])?;
    if !origin.starts_with("https://")
        || origin[8..].contains(['@', '?', '#'])
        || origin.chars().any(char::is_control)
    {
        return Err(
            "freshness checks require a public HTTPS origin without embedded credentials".into(),
        );
    }
    let dirty =
        !isolated_text(root, &["status", "--porcelain", "--untracked-files=normal"])?.is_empty();
    Ok(LocalSourceState {
        revision,
        dirty,
        current_branch,
        origin,
    })
}
pub fn parse_remote(text: &str, reference: &str) -> Result<String> {
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() != 1 {
        return Err("remote did not return exactly one selected ref".into());
    }
    let (hash, name) = lines[0].split_once('\t').ok_or("malformed remote ref")?;
    if !oid(hash) || name != reference {
        return Err("remote ref mismatch".into());
    }
    Ok(hash.into())
}
