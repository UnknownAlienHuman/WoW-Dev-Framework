//! Development driver for the documented native Ketho source-to-annotations lane.
//! Reads an exact local Git revision and its selected APIDocumentation TOC; never
//! executes source Lua or an external interpreter. This is not the public service-owned `wow` CLI.
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_reference::native::{NativeError, ingest_document, source_digest};

const LIMIT: usize = 1024 * 1024;
const TOTAL_LIMIT: usize = 64 * LIMIT;
const USAGE: &str = "native_library <git-checkout> <revision-or-ref> <generated-api.toc> <environment> <new-output-directory>";

#[derive(Serialize)]
struct Failure {
    path: String,
    sha256: String,
    error: NativeError,
}

fn main() -> std::process::ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(partial) => {
            if partial {
                std::process::ExitCode::from(3)
            } else {
                std::process::ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("native_library: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
fn run(args: Vec<OsString>) -> Result<bool, Box<dyn std::error::Error>> {
    if args.len() == 1 && args[0] == "--help" {
        println!("{USAGE}");
        return Ok(false);
    }
    if args.len() != 5 {
        return Err(USAGE.into());
    }
    let root = Path::new(&args[0]);
    let selector = args[1].to_str().ok_or("ref is not UTF-8")?;
    if selector.is_empty() || selector.starts_with('-') || selector.chars().any(char::is_control) {
        return Err("invalid source ref".into());
    }
    let resolved = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{selector}^{{commit}}"),
        ],
        128,
    )?;
    let revision = std::str::from_utf8(&resolved)?.trim();
    if !matches!(revision.len(), 40 | 64) || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid resolved revision".into());
    }
    let toc_path = args[2].to_str().ok_or("TOC path is not UTF-8")?;
    validate_path(toc_path)?;
    let environment = args[3].to_str().ok_or("environment is not UTF-8")?;
    if environment.is_empty() || environment.chars().any(char::is_control) {
        return Err("invalid source environment".into());
    }
    let toc_bytes = git(
        root,
        &["cat-file", "blob", &format!("{revision}:{toc_path}")],
        LIMIT,
    )?;
    let toc = std::str::from_utf8(&toc_bytes)?.trim_start_matches('\u{feff}');
    let parent = toc_path.rsplit_once('/').map_or("", |v| v.0);
    let mut paths = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for line in toc.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line.replace('\\', "/");
        validate_path(&name)?;
        if !name.ends_with(".lua") {
            return Err("selected TOC requires an unsupported load-entry form".into());
        }
        let path = if parent.is_empty() {
            name
        } else {
            format!("{parent}/{name}")
        };
        if !seen.insert(path.clone()) {
            return Err("selected TOC contains duplicate input paths".into());
        }
        paths.push(path);
        if paths.len() > 4096 {
            return Err("selected TOC exceeds file limit".into());
        }
    }
    if paths.is_empty() {
        return Err("selected TOC has no Lua inputs".into());
    }
    let cancelled = AtomicBool::new(false);
    let mut documents = Vec::new();
    let mut failures = Vec::new();
    let mut total = 0usize;
    for path in &paths {
        // Git object access ignores dirty files, filters, export-ignore and substitution.
        let bytes = git(
            root,
            &["cat-file", "blob", &format!("{revision}:{path}")],
            LIMIT,
        )?;
        total = total.checked_add(bytes.len()).ok_or("source byte limit")?;
        if total > TOTAL_LIMIT {
            return Err("source corpus exceeds byte limit".into());
        }
        let sha256 = source_digest(&bytes);
        let source = std::str::from_utf8(&bytes)?;
        match ingest_document(revision, path, source, &sha256, &cancelled) {
            Ok(document) => documents.push(document),
            Err(error) => failures.push(Failure {
                path: path.clone(),
                sha256,
                error,
            }),
        }
    }
    if documents.is_empty() {
        return Err("no source registrations could be admitted".into());
    }
    let library = wow_annotations::native::project(&documents, environment, &cancelled)?;
    let partial = !failures.is_empty() || !library.issues.is_empty();
    let report = serde_json::json!({
        "schema": "wow-native-source-build/1", "revision": revision, "selector": selector,
        "freshness": "not_network_verified", "environment": environment,
        "toc": {"path":toc_path,"sha256":source_digest(&toc_bytes)}, "source_order":paths,
        "candidate_files":paths.len(), "admitted_files":documents.len(), "input_failures":failures,
        "status": if partial {"partial"} else {"projected_with_sidecars"},
        "negative_authority":false, "library":library
    });
    // Refuse an existing output directory. Write only renderer-owned names under
    // a newly created private directory; a failed write is never complete output.
    // Publication/crash-recovery and multi-process root locking are higher owners.
    let report_bytes = serde_json::to_vec_pretty(&report)?;
    if report_bytes.len() > 512 * LIMIT {
        return Err("report exceeds byte limit".into());
    }
    let destination = Path::new(&args[4]);
    fs::create_dir(destination)?;
    for file in &library.files {
        write_new(&destination.join(&file.path), file.text.as_bytes())?;
    }
    write_new(&destination.join("source-report.json"), &report_bytes)?;
    println!(
        "{}",
        serde_json::json!({"revision":revision,"candidate_files":paths.len(),"admitted_files":documents.len(),"input_failures":failures.len(),"annotation_files":library.files.len(),"projection_issues":library.issues.len(),"status":if partial {"partial"} else {"projected_with_sidecars"},"negative_authority":false})
    );
    Ok(partial)
}
fn validate_path(path: &str) -> Result<(), &'static str> {
    if path.is_empty()
        || path.len() > 4096
        || path.contains([':', '\\'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
    {
        return Err("unsafe source-relative path");
    }
    Ok(())
}
fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
fn git(root: &Path, args: &[&str], limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut child = Command::new("git")
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "core.hooksPath=",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "submodule.recurse=false",
        ])
        .args(args)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_NAMESPACE")
        .env_remove("GIT_CONFIG_COUNT")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut bytes = Vec::new();
    let result = child
        .stdout
        .take()
        .ok_or("missing git output")?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes);
    if result.is_err() || bytes.len() > limit {
        let _ = child.kill();
        let _ = child.wait();
        result?;
        return Err("Git object exceeds input limit".into());
    }
    if !child.wait()?.success() {
        return Err(
            "Git could not read the selected local object; materialize missing objects explicitly"
                .into(),
        );
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let path = env::temp_dir().join(format!(
                "wow-native-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            let fixture = Self(path);
            fixture.command(&["init", "--initial-branch=main"])?;
            fixture.command(&["config", "user.name", "Native Test"])?;
            fixture.command(&["config", "user.email", "test@example.invalid"])?;
            fs::write(fixture.0.join("API.toc"), "API.lua\n")?;
            fs::write(
                fixture.0.join("API.lua"),
                r#"local s={Name="Example",Type="System",Namespace="C_Example",Functions={{Name="Read",Returns={{Name="value",Type="bool"}}}}} APIDocumentation:AddDocumentationTable(s)"#,
            )?;
            fixture.command(&["add", "."])?;
            fixture.command(&["commit", "-m", "fixture"])?;
            Ok(fixture)
        }
        fn command(&self, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
            if !Command::new("git")
                .arg("-C")
                .arg(&self.0)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?
                .success()
            {
                return Err("test git setup failed".into());
            }
            Ok(())
        }
        fn args(&self, output: &str) -> Vec<OsString> {
            vec![
                self.0.clone().into_os_string(),
                "HEAD".into(),
                "API.toc".into(),
                "Mainline".into(),
                self.0.join(output).into_os_string(),
            ]
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn exact_git_snapshot_not_dirty_worktree_and_no_output_clobber()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        fs::write(
            fixture.0.join("API.lua"),
            "os.execute('untrusted worktree')",
        )?;
        assert!(!run(fixture.args("out"))?);
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture.0.join("out/source-report.json"))?)?;
        assert_eq!(report["admitted_files"], 1);
        assert_eq!(report["negative_authority"], false);
        assert!(
            fs::read_to_string(fixture.0.join("out/api-0000.lua"))?
                .contains("function C_Example.Read() end")
        );
        assert!(run(fixture.args("out")).is_err());
        Ok(())
    }
    #[test]
    fn unsupported_file_is_an_explicit_partial_not_missing_success()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        fs::write(fixture.0.join("Bad.lua"), "os.execute('never execute')")?;
        fs::write(fixture.0.join("API.toc"), "API.lua\nBad.lua\n")?;
        fixture.command(&["add", "."])?;
        fixture.command(&["commit", "-m", "unsupported fixture"])?;
        assert!(run(fixture.args("out"))?);
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture.0.join("out/source-report.json"))?)?;
        assert_eq!(report["candidate_files"], 2);
        assert_eq!(report["admitted_files"], 1);
        assert_eq!(report["input_failures"].as_array().map(Vec::len), Some(1));
        Ok(())
    }
    #[test]
    fn toc_traversal_rejects_before_output_creation() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        fs::write(fixture.0.join("API.toc"), "../API.lua\n")?;
        fixture.command(&["add", "."])?;
        fixture.command(&["commit", "-m", "unsafe fixture"])?;
        assert!(run(fixture.args("out")).is_err());
        assert!(!fixture.0.join("out").exists());
        Ok(())
    }
}
