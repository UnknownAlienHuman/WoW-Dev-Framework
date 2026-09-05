use crate::Result;
use std::fs;
use std::path::Path;

pub fn forbidden_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or("");
    [
        ".py", ".pyw", ".pyc", ".pyo", ".pyi", ".pyx", ".pxd", ".ipynb",
    ]
    .iter()
    .any(|ext| lower.ends_with(ext))
        || matches!(
            name,
            "pyproject.toml"
                | "pipfile"
                | "pipfile.lock"
                | "poetry.lock"
                | "uv.lock"
                | ".python-version"
                | "setup.py"
                | "tox.ini"
                | "pytest.ini"
        )
        || (name.starts_with("requirements") && name.ends_with(".txt"))
        || lower
            .split('/')
            .any(|p| matches!(p, "__pycache__" | ".venv" | "venv" | ".pytest_cache"))
}
/// Source policy checks catch ordinary reintroduction, not deliberately obfuscated
/// programs. Prose describing the prohibition is allowed; executable use is not.
pub fn forbidden_content(path: &str, text: &str) -> bool {
    if text
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("#!") && line.to_ascii_lowercase().contains("python"))
    {
        return true;
    }
    let lower = text.to_ascii_lowercase();
    if path.ends_with(".rs") {
        return lower.contains("command::new(\"python")
            || lower.lines().any(|line| {
                let line = line.trim_start();
                line.starts_with("use pyo3") || line.starts_with("use rustpython")
            });
    }
    if path.ends_with(".toml") || path == "Cargo.lock" {
        return lower.contains("pyo3") || lower.contains("rustpython");
    }
    let executable = path.ends_with(".sh")
        || path.ends_with(".ps1")
        || path.ends_with(".yml")
        || path.ends_with(".yaml")
        || path.ends_with(".toml")
        || path == "Makefile";
    if !executable {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    [
        "actions/setup-python",
        "python -",
        "python3 -",
        "python scripts/",
        "python3 scripts/",
        "pip install",
        "pip3 install",
        "python -m",
        "python3 -m",
        "command::new(\"python",
        "pyo3",
        "rustpython",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern))
}
pub fn check(root: &Path) -> Result<()> {
    let listing = crate::git::run(
        root,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        None,
        16 * 1024 * 1024,
    )?;
    let mut count = 0usize;
    for raw in listing.split(|b| *b == 0).filter(|entry| !entry.is_empty()) {
        let relative = std::str::from_utf8(raw)?;
        crate::manifest::validate_path(relative)?;
        if forbidden_path(relative) {
            return Err(format!("forbidden interpreter asset: {relative}").into());
        }
        let file = root.join(relative);
        let meta = fs::symlink_metadata(&file)?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(format!("non-regular tracked file: {relative}").into());
        }
        if meta.len() > 8 * 1024 * 1024 {
            return Err("repository check file limit exceeded".into());
        }
        let bytes = fs::read(file)?;
        if let Ok(text) = std::str::from_utf8(&bytes) {
            if forbidden_content(relative, text) {
                return Err(format!("forbidden interpreter invocation: {relative}").into());
            }
            if relative.ends_with(".json") {
                let _: serde_json::Value = serde_json::from_str(text)?;
            }
        } else if relative.ends_with(".json") {
            return Err(format!("non-UTF-8 JSON: {relative}").into());
        }
        count += 1;
    }
    if count == 0 {
        return Err("empty repository inventory".into());
    }
    println!("Checked {count} distributable files; native-only policy passed");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_python_assets_in_any_directory_and_case() {
        for name in [
            "a.py",
            "tests/A.PYW",
            "stub.pyi",
            "compiled.pyc",
            "notebook.ipynb",
            "pkg/pyproject.toml",
            "uv.lock",
            "pkg/requirements-dev.txt",
            ".venv/config",
            "a/__pycache__/x",
        ] {
            assert!(forbidden_path(name), "{name}");
        }
        for name in [
            "src/lib.rs",
            "docs/PIPELINE.md",
            "Cargo.lock",
            ".cargo/config.toml",
        ] {
            assert!(!forbidden_path(name), "{name}");
        }
    }
    #[test]
    fn rejects_extensionless_shebangs_ci_steps_and_rust_subprocesses() {
        for (path, text) in [
            ("runner", "#!/usr/bin/env python3\n"),
            ("ci.yml", "uses: actions/setup-python@v7"),
            ("src/x.rs", "Command::new(\"python3\")"),
            ("test.sh", "python -m unittest"),
            ("Cargo.toml", "pyo3 = '0.1'"),
        ] {
            assert!(forbidden_content(path, text));
        }
        assert!(!forbidden_content(
            "README.md",
            "No Python code is permitted."
        ));
        assert!(!forbidden_content("ci.yml", "run: cargo test --workspace"));
    }
}
