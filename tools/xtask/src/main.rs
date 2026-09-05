//! Repository maintenance only; not an alternate product/service implementation.
mod git;
mod library;
mod manifest;
mod repository;
mod skill;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const USAGE: &str = "cargo xtask check [--root DIR]\ncargo xtask sync-skill --check|--write [--root DIR]\ncargo xtask check-source CHECKOUT BRANCH\ncargo xtask manifest CHECKOUT REF SELECTOR OUTPUT\ncargo xtask verify-manifest MANIFEST CHECKOUT [CURRENT_REF]\ncargo xtask verify-library OUTPUT [--require-input-complete]";
fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::from(2)
        }
    }
}
fn run(args: Vec<std::ffi::OsString>) -> Result<u8> {
    let strings = args
        .iter()
        .map(|v| v.to_str().ok_or("arguments must be UTF-8"))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    match strings.as_slice() {
        [] | ["--help"] | ["help"] => {
            println!("{USAGE}");
            Ok(0)
        }
        ["check", rest @ ..] => {
            let root = repository_root(rest)?;
            repository::check(&root)?;
            if !skill::sync(&root, false)? {
                return Ok(3);
            }
            println!("Repository checks passed");
            Ok(0)
        }
        ["sync-skill", mode @ ("--check" | "--write"), rest @ ..] => Ok(
            if skill::sync(&repository_root(rest)?, *mode == "--write")? {
                0
            } else {
                3
            },
        ),
        ["check-source", root, branch] => git::check_source(Path::new(root), branch),
        ["manifest", root, revision, selector, output] => {
            let value = manifest::build(Path::new(root), revision, selector)?;
            manifest::write_new(Path::new(output), &value)?;
            println!(
                "{}",
                serde_json::json!({"status":"verified_inventory", "source":value["source"], "coverage":value["coverage"], "manifest_sha256":value["manifest_sha256"]})
            );
            Ok(0)
        }
        ["verify-manifest", file, root, rest @ ..] if rest.len() <= 1 => {
            manifest::verify(Path::new(file), Path::new(root), rest.first().copied())
        }
        ["verify-library", root] => library::verify(Path::new(root), false),
        ["verify-library", root, "--require-input-complete"] => {
            library::verify(Path::new(root), true)
        }
        _ => Err(USAGE.into()),
    }
}
fn repository_root(args: &[&str]) -> Result<PathBuf> {
    match args {
        [] => Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?),
        ["--root", root] => Ok(Path::new(root).canonicalize()?),
        _ => Err(USAGE.into()),
    }
}
