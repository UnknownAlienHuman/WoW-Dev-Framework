//! Record the executable and (for LuaLS) its shipped Lua implementation/data.
//! Hashes identify inspected bytes, not authorship or a platform sandbox.
use super::{
    Result,
    process::{Consumer, Executable},
};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
use wow_reference::native::source_digest;
pub fn snapshot(exe: &Executable) -> Result<(usize, String)> {
    let mut files = BTreeMap::new();
    let mut total = 0usize;
    match exe.kind {
        Consumer::Emmy => {
            files.insert("executable".into(), exe.digest.clone());
        }
        Consumer::LuaLs => {
            let root = exe
                .path
                .parent()
                .and_then(Path::parent)
                .ok_or("LuaLS must retain its shipped bin/ layout")?;
            visit(root, root, &mut files, &mut total)?;
        }
    }
    Ok((files.len(), source_digest(&serde_json::to_vec(&files)?)))
}
fn visit(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, String>,
    total: &mut usize,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            return Err("consumer package symlink rejected".into());
        }
        if meta.is_dir() {
            if path.strip_prefix(root)?.components().count() > 24 {
                return Err("consumer package depth exceeded".into());
            }
            visit(root, &path, files, total)?;
        } else if meta.is_file() {
            if files.len() >= 20_000 || meta.len() > 128 * 1024 * 1024 {
                return Err("consumer package limit".into());
            }
            let mut bytes = Vec::new();
            fs::File::open(&path)?
                .take(128 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)?;
            *total = total.checked_add(bytes.len()).ok_or("package byte limit")?;
            if *total > 256 * 1024 * 1024 || bytes.len() > 128 * 1024 * 1024 {
                return Err("package byte limit".into());
            }
            let name = path
                .strip_prefix(root)?
                .to_str()
                .ok_or("invalid consumer package path")?
                .replace('\\', "/");
            files.insert(name, source_digest(&bytes));
        } else {
            return Err("nonregular consumer package file".into());
        }
    }
    Ok(())
}
