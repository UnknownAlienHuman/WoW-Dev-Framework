use crate::Result;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const CANONICAL: &str = ".agents/skills/wow-dev/SKILL.md";
pub const TARGETS: [&str; 3] = [
    ".claude/skills/wow-dev/SKILL.md",
    ".opencode/skills/wow-dev/SKILL.md",
    ".agent/skills/wow-dev/SKILL.md",
];
const LIMIT: usize = 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct RemoveFile(PathBuf);
impl Drop for RemoveFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Refuse symlinks in any component, including dangling links. Preflight all
/// targets before writing; assume a trusted checkout, not an adversarial host.
fn path(root: &Path, relative: &str) -> Result<PathBuf> {
    crate::manifest::validate_path(relative)?;
    let mut current = root.to_path_buf();
    for part in relative.split('/') {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("skill path contains a symbolic link".into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(current)
}
fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !file.metadata()?.is_file() {
        return Err("skill must be a regular file".into());
    }
    let mut bytes = Vec::new();
    (&mut file).take(LIMIT as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > LIMIT {
        return Err("skill exceeds byte limit".into());
    }
    Ok(Some(bytes))
}
pub fn sync(root: &Path, write: bool) -> Result<bool> {
    let root = root.canonicalize()?;
    let canonical = read(&path(&root, CANONICAL)?)?.ok_or("missing canonical skill")?;
    let text = std::str::from_utf8(&canonical)?;
    if canonical.is_empty() || !text.starts_with("---\n") || !text.contains("name: wow-dev") {
        return Err("invalid canonical WoW skill".into());
    }
    let paths = TARGETS
        .iter()
        .map(|relative| path(&root, relative))
        .collect::<Result<Vec<_>>>()?;
    let mut states = paths.iter().map(|p| read(p)).collect::<Result<Vec<_>>>()?;
    if write {
        let lock_path = root.join(".wow-dev-skill.lock");
        let lock = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)?;
        let _cleanup = RemoveFile(lock_path);
        let _lock = lock;
        for (target, old) in paths.iter().zip(states.iter_mut()) {
            if old.as_ref() == Some(&canonical) {
                continue;
            }
            let parent = target.parent().ok_or("invalid skill target")?;
            fs::create_dir_all(parent)?;
            let temporary = parent.join(format!(
                ".skill-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary)?;
            let _cleanup = RemoveFile(temporary.clone());
            file.write_all(&canonical)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, target)?;
            *old = read(target)?;
        }
    }
    let mut current = true;
    for (target, bytes) in TARGETS.iter().zip(states.iter()) {
        let status = if bytes.as_ref() == Some(&canonical) {
            "current"
        } else if bytes.is_none() {
            "missing"
        } else {
            "stale"
        };
        current &= status == "current";
        println!("{status}: {target}");
    }
    if !current {
        eprintln!("Run cargo xtask sync-skill --write to update discovery copies");
    }
    Ok(current)
}
