//! Test-owned Git/TOC input for the real native driver, not another extractor.
use super::{Result, source_report};
use serde_json::Value;
use std::{collections::BTreeSet, ffi::OsString, fs, path::Path, process::Command};

#[path = "../../examples/support/mod.rs"]
pub(super) mod driver;

const TOC: &str = "API.toc";

pub(super) fn prepare(output: &Path, sources: &[(&str, &str)]) -> Result<()> {
    if sources.is_empty() || sources.len() > 8 {
        return Err("invalid synthetic source count".into());
    }
    let mut names = BTreeSet::new();
    for (name, _) in sources {
        if !name.ends_with(".lua")
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || name.starts_with('.')
            || name.starts_with('-')
            || !names.insert(*name)
        {
            return Err("invalid synthetic source path".into());
        }
    }
    let checkout = output.with_extension("source");
    fs::create_dir(&checkout)?;
    fs::create_dir(checkout.join("home"))?;
    fs::create_dir(checkout.join("hooks"))?;
    fs::write(checkout.join("home/.gitconfig"), "")?;
    git(&checkout, &["init", "--initial-branch=main", "--template="])?;
    let toc = sources
        .iter()
        .map(|(name, _)| format!("{name}\n"))
        .collect::<String>();
    fs::write(checkout.join(TOC), &toc)?;
    for (name, text) in sources {
        fs::write(checkout.join(name), text)?;
    }
    let mut paths = vec!["add", "--", TOC];
    paths.extend(sources.iter().map(|(name, _)| *name));
    git(&checkout, &paths)?;
    git(
        &checkout,
        &["commit", "--no-gpg-sign", "-m", "synthetic consumer source"],
    )?;
    let revision = git(&checkout, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let revision = revision.trim();
    if !matches!(revision.len(), 40 | 64) || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid synthetic source revision".into());
    }

    // The production driver must read committed blobs, not these dirty files.
    fs::write(checkout.join(TOC), "uncommitted.lua\n")?;
    fs::write(checkout.join(sources[0].0), "not committed documentation\n")?;
    let partial = driver::run(
        vec![
            checkout.into_os_string(),
            "HEAD".into(),
            TOC.into(),
            "Mainline".into(),
            output.as_os_str().to_owned(),
        ],
        None,
    )?;
    if partial {
        return Err("synthetic Git/TOC source build is partial".into());
    }
    let report: Value = serde_json::from_slice(&fs::read(output.join("source-report.json"))?)?;
    source_report::validate(&report, output, revision, sources)
}

pub(super) fn git(checkout: &Path, args: &[&str]) -> Result<String> {
    let mut command = Command::new("git");
    command.env_clear();
    for name in ["PATH", "SystemRoot", "WINDIR", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    let mut hooks = OsString::from("core.hooksPath=");
    hooks.push(checkout.join("hooks"));
    command
        .env("HOME", checkout.join("home"))
        .env("USERPROFILE", checkout.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", checkout.join("home/.gitconfig"))
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("LC_ALL", "C")
        .arg("-C")
        .arg(checkout)
        .arg("-c")
        .arg(hooks)
        .args([
            "-c",
            "core.autocrlf=false",
            "-c",
            "commit.gpgSign=false",
            "-c",
            "user.name=Consumer Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
        ])
        .args(args);
    let output = command.output()?;
    if !output.status.success() || output.stdout.len() > 4096 || output.stderr.len() > 4096 {
        return Err("test-owned Git source setup failed".into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
