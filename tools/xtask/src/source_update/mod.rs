//! Explicit source-data update. No source library or Wasm guest gains host IO.
mod lock;
mod remote;
mod state;
#[cfg(test)]
mod tests;

use crate::{Result, git};
use remote::Remote;
use serde_json::{Value, json};
use std::path::Path;

pub fn run(root: &Path, branch: &str, expected: &str) -> Result<u8> {
    let (code, report) = update(root, branch, expected, &remote::Https)?;
    println!("{report}");
    Ok(code)
}
fn report(status: &str, before: &str, selected: Option<&str>, after: Option<&str>) -> Value {
    json!({"schema":"wow-source-update/1", "status":status,
        "before_revision":before, "selected_remote_revision":selected, "after_revision":after,
        "freshness":if selected.is_some() { "selected_remote_observation_only" } else { "unverified_current" },
        "semantic_compatibility":"not_evaluated"})
}

fn update(root: &Path, branch: &str, expected: &str, remote: &impl Remote) -> Result<(u8, Value)> {
    if !git::oid(expected) {
        return Err("source update requires an exact expected HEAD".into());
    }
    let root = state::root(root)?;
    git::isolated_text(
        &root,
        &["check-ref-format", &format!("refs/heads/{branch}")],
    )?;
    let mut lock = lock::Lock::acquire(&root, expected)?;
    let before = state::inspect(&root)?;
    if before.head != expected || before.branch != branch {
        return Err("source HEAD or branch differs from the explicit update guard".into());
    }
    let selected = match remote.head(&root, &before.origin, branch) {
        Ok(selected) if git::oid(&selected) && selected.len() == expected.len() => selected,
        _ => {
            lock.release()?;
            return Ok((4, report("unverified_current", expected, None, None)));
        }
    };
    if state::inspect(&root)? != before {
        return Err("source state changed during remote observation; update refused".into());
    }
    if selected == expected {
        lock.release()?;
        return Ok((
            0,
            report("current", expected, Some(&selected), Some(expected)),
        ));
    }
    if remote.fetch(&root, &before.origin, &selected).is_err() {
        lock.release()?;
        let mut result = report("fetch_failed", expected, Some(&selected), None);
        result["object_database_may_have_changed"] = json!(true);
        return Ok((4, result));
    }
    // Revalidate after network IO, before any checkout/ref mutation.
    if state::inspect(&root)? != before {
        return Err("source state changed during fetch; checkout update refused".into());
    }
    if state::resolve(&root, &selected)? != selected {
        return Err("fetched source identity mismatch".into());
    }
    if git::isolated_text(&root, &["merge-base", "--is-ancestor", expected, &selected]).is_err() {
        lock.release()?;
        return Ok((
            3,
            report(
                "not_fast_forward_or_incomplete_history",
                expected,
                Some(&selected),
                Some(expected),
            ),
        ));
    }
    lock.prepare(&selected)?;
    // Never reset/rebase/stash/create a merge commit. Refuse ignored overwrites.
    // Git hooks, inherited config and lazy fetching are disabled by the adapter.
    let applied = git::isolated_text(
        &root,
        &[
            "-c",
            "submodule.recurse=false",
            "-c",
            "merge.autoStash=false",
            "-c",
            "merge.verifySignatures=false",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
            "merge",
            "--ff-only",
            "--no-edit",
            "--no-stat",
            "--no-autostash",
            "--no-verify-signatures",
            "--no-overwrite-ignore",
            "--",
            &selected,
        ],
    );
    let observed = state::inspect(&root);
    if let (Ok(_), Ok(after)) = (&applied, &observed)
        && after.head == selected
        && after.branch == before.branch
        && after.origin == before.origin
        && after.config_hash == before.config_hash
        && lock.release().is_ok()
    {
        return Ok((
            0,
            report("updated", expected, Some(&selected), Some(&selected)),
        ));
    }
    // A failed Git command may have touched index/worktree/ORIG_HEAD. Even when
    // HEAD looks unchanged, do not retry or reset. Keep the exact journal/lock.
    let mut result = report(
        "reconciliation_required",
        expected,
        Some(&selected),
        observed.as_ref().ok().map(|s| s.head.as_str()),
    );
    result["lock_retained"] = json!(true);
    Ok((5, result))
}
