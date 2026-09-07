//! Explicit development composition; VM dependencies stay outside stable owners.
//! Reuses the canonical Git/TOC driver, including corrections and alias catalogs.
#[path = "../../../crates/wow-annotations/examples/support/mod.rs"]
mod driver;
#[path = "support/limits_args.rs"]
mod limits_args;
use std::{fs, io::Read, path::Path, process::ExitCode};
use wow_literal_host::{ModuleHandle, ModuleSlot, ObservedSnapshot};
const USAGE: &str = "source_library <module.wasm> <approved-sha256:...> [--fuel N] [--memory-bytes N] <checkout> <ref> <TOC> <environment> <new-output> [--corrections ...] [--alias-catalog ...]...";
fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()) {
        Ok(partial) => ExitCode::from(if partial { 3 } else { 0 }),
        Err(error) => {
            eprintln!("source_library: {error}");
            ExitCode::from(2)
        }
    }
}
fn run(args: Vec<std::ffi::OsString>) -> Result<bool, Box<dyn std::error::Error>> {
    if args.len() == 1 && args[0] == "--help" {
        println!("{USAGE}");
        return Ok(false);
    }
    if args.len() < 7 {
        return Err(USAGE.into());
    }
    let (limits, used) = limits_args::parse(&args[2..])?;
    if args.len() - used < 7 {
        return Err(USAGE.into());
    }
    let path = Path::new(&args[0]);
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 8 * 1024 * 1024
    {
        return Err("module must be a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    let approved = args[1].to_str().ok_or("approved digest must be UTF-8")?;
    let slot = ModuleSlot::new(ModuleHandle::load(&bytes, approved, limits)?);
    let observed = ObservedSnapshot::new(slot.snapshot()?);
    let result = driver::run(args[2 + used..].to_vec(), Some(&observed));
    // Print only fixed host classes and numeric counters, never source/VM text.
    eprintln!(
        "literal limits={:?} successful-call usage={:?}",
        observed.limits(),
        observed.usage()
    );
    if let Some(error) = observed.failure() {
        return Err(error.into());
    }
    result
}
