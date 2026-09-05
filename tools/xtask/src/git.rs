use crate::Result;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Fixed Git subprocesses: no shell, no inherited Git root overrides, bounded
/// output and deadline. Stderr is discarded: it can contain credentialed URLs.
pub fn run(root: &Path, args: &[&str], input: Option<Vec<u8>>, limit: usize) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
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
    text(root, &["check-ref-format", &format!("refs/heads/{branch}")])?;
    let revision = resolve(root, "HEAD")?;
    let dirty = !text(root, &["status", "--porcelain", "--untracked-files=normal"])?.is_empty();
    let current_branch = text(root, &["symbolic-ref", "--quiet", "--short", "HEAD"]).ok();
    let origin = text(root, &["remote", "get-url", "origin"])?;
    if !origin.starts_with("https://")
        || origin[8..].contains(['@', '?', '#'])
        || origin.chars().any(char::is_control)
    {
        return Err(
            "freshness checks require a public HTTPS origin without embedded credentials".into(),
        );
    }
    let reference = format!("refs/heads/{branch}");
    let remote = text(
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
