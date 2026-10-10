//! Explicit read-only Git object acquisition for the compiled probe executable.
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::AtomicBool,
};
use wow_emmy::{
    EMMYLUA_REVISION, EMMYLUA_TREE,
    compatibility::compiled_probe::{
        COMPILED_PROBE_MEMBER_MAX_BYTES, COMPILED_PROBE_MEMBERS, probe_compiled_backend,
    },
};

pub(super) fn produce(checkout: &Path) -> Result<Vec<u8>, String> {
    let tree = git(
        checkout,
        &["rev-parse", &format!("{EMMYLUA_REVISION}^{{tree}}")],
        128,
    )?;
    if std::str::from_utf8(&tree)
        .map_err(|_| "invalid source tree identity")?
        .trim()
        != EMMYLUA_TREE
    {
        return Err("upstream Git tree differs from the compiled backend".to_owned());
    }
    let mut members = Vec::with_capacity(COMPILED_PROBE_MEMBERS.len());
    for path in COMPILED_PROBE_MEMBERS {
        let entry = git(checkout, &["ls-tree", EMMYLUA_REVISION, "--", path], 1024)?;
        let entry = std::str::from_utf8(&entry).map_err(|_| "invalid upstream member entry")?;
        let (metadata, retained_path) = entry
            .trim_end()
            .split_once('\t')
            .ok_or("missing upstream source member")?;
        if !metadata.starts_with("100644 blob ") || retained_path != *path {
            return Err("upstream probe member is not an exact regular Git blob".to_owned());
        }
        members.push(git(
            checkout,
            &["cat-file", "blob", &format!("{EMMYLUA_REVISION}:{path}")],
            COMPILED_PROBE_MEMBER_MAX_BYTES,
        )?);
    }
    probe_compiled_backend(
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        &members,
        &AtomicBool::new(false),
    )
    .map_err(|error| {
        format!(
            "compiled backend probe failed ({:?}): {}",
            error.code(),
            error.message()
        )
    })
}

fn git(checkout: &Path, arguments: &[&str], limit: usize) -> Result<Vec<u8>, String> {
    let mut child = Command::new("git")
        .args(["--no-replace-objects", "--no-optional-locks", "-C"])
        .arg(checkout)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "cannot start read-only Git object reader".to_owned())?;
    let result = (|| {
        let mut bytes = Vec::new();
        child
            .stdout
            .take()
            .ok_or("Git object reader has no stdout")?
            .take((limit as u64) + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "cannot read upstream Git object")?;
        if bytes.len() > limit {
            return Err("upstream Git object exceeds the probe read limit".to_owned());
        }
        Ok(bytes)
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child
        .wait()
        .map_err(|_| "cannot reap Git object reader".to_owned())?;
    if !status.success() {
        return Err("exact upstream Git object read failed".to_owned());
    }
    result
}
