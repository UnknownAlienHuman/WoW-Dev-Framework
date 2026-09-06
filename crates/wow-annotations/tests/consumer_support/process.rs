//! Explicit approved binaries, fixed command shapes, no shell or config discovery.
use super::Result;
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use wow_reference::native::source_digest;
const LIMIT: u64 = 8 * 1024 * 1024;
#[derive(Clone, Copy, Debug)]
pub enum Consumer {
    Emmy,
    LuaLs,
}
impl Consumer {
    pub fn name(self) -> &'static str {
        match self {
            Self::Emmy => "emmy",
            Self::LuaLs => "luals",
        }
    }
    fn env(self) -> &'static str {
        match self {
            Self::Emmy => "WDF_EMMY_CHECK",
            Self::LuaLs => "WDF_LUALS",
        }
    }
}
pub struct Executable {
    pub kind: Consumer,
    pub path: PathBuf,
    pub digest: String,
}
impl Executable {
    pub fn approved(kind: Consumer) -> Result<Self> {
        let path =
            PathBuf::from(std::env::var_os(kind.env()).ok_or("consumer binary not supplied")?);
        if !path.is_absolute() {
            return Err("consumer binary must have an explicit absolute path".into());
        }
        let digest = hash_executable(&path)?;
        let expected = std::env::var(format!("{}_SHA256", kind.env()))
            .map_err(|_| "approved consumer hash not supplied")?;
        if expected != digest {
            return Err("consumer binary hash mismatch".into());
        }
        Ok(Self { kind, path, digest })
    }
    pub fn command(&self, root: &Path) -> Command {
        let mut command = Command::new(&self.path);
        command.env_clear().current_dir(root).stdin(Stdio::null());
        for key in ["SystemRoot", "WINDIR"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        let home = root.join("home");
        for key in [
            "HOME",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
            "TMP",
            "TEMP",
            "TMPDIR",
        ] {
            command.env(key, &home);
        }
        // LuaLS initializes its logger even for --version. Route every invocation,
        // not only diagnosis, away from the approved package and user directories.
        if matches!(self.kind, Consumer::LuaLs) {
            command
                .arg("--logpath")
                .arg(root.join("logs"))
                .arg("--metapath")
                .arg(root.join("meta"));
        }
        command.env("LANG", "C.UTF-8");
        command
    }
    pub fn version(&self, root: &Path) -> Result<String> {
        let mut command = self.command(root);
        command.arg("--version");
        let prefix = format!("{}-version", self.kind.name());
        if execute(command, root, &prefix, None)? != 0 {
            return Err("consumer version failed".into());
        }
        let text = String::from_utf8(read(&root.join(format!("{prefix}.out")))?)?;
        let text = text.trim();
        if text.is_empty() || text.len() > 256 || text.chars().any(char::is_control) {
            return Err("invalid consumer version".into());
        }
        Ok(text.into())
    }
    pub fn check(&self, root: &Path) -> Result<(i32, Vec<u8>)> {
        let input = root.join("input");
        let output = root.join(format!("{}.json", self.kind.name()));
        if output.exists() {
            return Err("consumer output already exists".into());
        }
        if hash_executable(&self.path)? != self.digest {
            return Err("consumer binary changed before invocation".into());
        }
        let mut command = self.command(root);
        match self.kind {
            Consumer::Emmy => {
                command
                    .arg(&input)
                    .arg("--config")
                    .arg(input.join("emmy.json"))
                    .args(["--output-format", "json", "--output"])
                    .arg(&output)
                    .arg("--warnings-as-errors");
            }
            Consumer::LuaLs => {
                command
                    .arg("--check")
                    .arg(&input)
                    .arg("--configpath")
                    .arg(input.join("luals.json"))
                    .args([
                        "--checklevel=Hint",
                        "--check_format=json",
                        "--check_out_path",
                    ])
                    .arg(&output);
            }
        }
        let status = execute(command, root, self.kind.name(), Some(&output))?;
        if ![0, 1].contains(&status) {
            return Err("consumer exited abnormally".into());
        }
        if hash_executable(&self.path)? != self.digest {
            return Err("consumer binary changed".into());
        }
        Ok((status, read(&output)?))
    }
}
fn hash_executable(path: &Path) -> Result<String> {
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > 128 * 1024 * 1024 {
        return Err("invalid consumer binary".into());
    }
    Ok(source_digest(&fs::read(path)?))
}
pub fn read(path: &Path) -> Result<Vec<u8>> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > LIMIT {
        return Err("invalid consumer output".into());
    }
    let mut bytes = Vec::new();
    File::open(path)?.take(LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err("consumer output limit".into());
    }
    Ok(bytes)
}
fn execute(mut command: Command, root: &Path, label: &str, report: Option<&Path>) -> Result<i32> {
    let stdout = root.join(format!("{label}.out"));
    let stderr = root.join(format!("{label}.err"));
    command
        .stdout(File::create_new(&stdout)?)
        .stderr(File::create_new(&stderr)?);
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut owned = ChildGuard(command.spawn()?);
    let child = &mut owned.0;
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let oversized = [Some(stdout.as_path()), Some(stderr.as_path()), report]
            .into_iter()
            .flatten()
            .any(|p| fs::metadata(p).is_ok_and(|m| m.len() > LIMIT));
        if oversized || Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("consumer deadline/output bound exceeded".into());
        }
        if let Some(status) = child.try_wait()? {
            return status
                .code()
                .ok_or_else(|| "consumer terminated by signal".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
