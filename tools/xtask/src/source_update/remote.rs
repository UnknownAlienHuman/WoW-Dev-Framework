//! Fixed network bridge. The checkout/provider is explicit, never source-derived.
use crate::{Result, git};
use std::path::Path;

pub(super) trait Remote {
    fn head(&self, root: &Path, origin: &str, branch: &str) -> Result<String>;
    fn fetch(&self, root: &Path, origin: &str, revision: &str) -> Result<()>;
}

pub(super) struct Https;
impl Remote for Https {
    fn head(&self, root: &Path, origin: &str, branch: &str) -> Result<String> {
        validate_origin(origin)?;
        let reference = format!("refs/heads/{branch}");
        let output = git::isolated_text(
            root,
            &[
                "-c",
                "credential.helper=",
                "-c",
                "core.askPass=",
                "-c",
                "http.followRedirects=false",
                "-c",
                "protocol.allow=never",
                "-c",
                "protocol.https.allow=always",
                "ls-remote",
                "--exit-code",
                "--refs",
                origin,
                &reference,
            ],
        )?;
        git::parse_remote(&output, &reference)
    }
    fn fetch(&self, root: &Path, origin: &str, revision: &str) -> Result<()> {
        validate_origin(origin)?;
        if !git::oid(revision) {
            return Err("invalid selected source revision".into());
        }
        // Observe a moving ref once, then fetch only that exact commit. No local
        // branch, tracking ref, FETCH_HEAD, tag or submodule update is requested.
        git::isolated_text(
            root,
            &[
                "-c",
                "credential.helper=",
                "-c",
                "core.askPass=",
                "-c",
                "http.followRedirects=false",
                "-c",
                "protocol.allow=never",
                "-c",
                "protocol.https.allow=always",
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                "--no-recurse-submodules",
                "--no-auto-maintenance",
                "--refmap=",
                origin,
                revision,
            ],
        )?;
        Ok(())
    }
}

pub(super) fn validate_origin(origin: &str) -> Result<()> {
    let tail = origin
        .strip_prefix("https://")
        .ok_or("source origin must use HTTPS")?;
    if tail.split('/').next().is_none_or(str::is_empty)
        || tail.contains(['@', '?', '#', '\\'])
        || origin.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err("invalid or credential-bearing source origin".into());
    }
    Ok(())
}
