use crate::Result;
use std::collections::BTreeSet;
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

/// Validate syntax first, then reject duplicate decoded names in each object.
///
/// A serde_json::Value alone silently replaces repeated object members. Keep
/// serde_json as the syntax, string-escape, number and recursion-limit authority;
/// this second, iterative walk only tracks object scopes and member names. It
/// does not interpret values or narrow the existing arbitrary-precision numbers.
/// The caller's file-size bound also bounds retained names. Error messages do
/// not echo member names or values, only their source byte offset.
fn validate_json(text: &str) -> Result<()> {
    let _: serde_json::Value = serde_json::from_str(text)?;
    let bytes = text.as_bytes();
    let mut objects: Vec<BTreeSet<String>> = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'{' => {
                objects.push(BTreeSet::new());
                cursor += 1;
            }
            b'}' => {
                if objects.pop().is_none() {
                    return Err("JSON object scope mismatch".into());
                }
                cursor += 1;
            }
            b'"' => {
                let start = cursor;
                cursor += 1;
                loop {
                    match bytes.get(cursor) {
                        Some(b'\\') => cursor += 2,
                        Some(b'"') => {
                            cursor += 1;
                            break;
                        }
                        Some(_) => cursor += 1,
                        None => return Err("unterminated JSON string".into()),
                    }
                }
                let mut next = cursor;
                while bytes.get(next).is_some_and(u8::is_ascii_whitespace) {
                    next += 1;
                }
                if bytes.get(next) == Some(&b':') {
                    let key: String = serde_json::from_str(&text[start..cursor])?;
                    let object = objects.last_mut().ok_or("JSON key outside object")?;
                    if !object.insert(key) {
                        return Err(format!(
                            "duplicate JSON object key at UTF-8 byte {start}"
                        )
                        .into());
                    }
                }
            }
            _ => cursor += 1,
        }
    }
    Ok(())
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
                validate_json(text)
                    .map_err(|error| format!("invalid JSON in {relative}: {error}"))?;
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

    #[test]
    fn value_parsing_alone_does_not_prove_unique_names() -> Result<()> {
        let text = r#"{"implemented":false,"implemented":true}"#;
        let parsed: serde_json::Value = serde_json::from_str(text)?;
        assert_eq!(parsed["implemented"], true);
        assert!(validate_json(text).is_err());
        Ok(())
    }

    #[test]
    fn rejects_duplicate_keys_in_nested_objects_and_arrays() {
        for text in [
            r#"{"a":1,"b":2,"a":3}"#,
            r#"{"outer":{"a":1,"a":2}}"#,
            r#"[{"a":1,"a":2}]"#,
            r#"{"a":[[{"b":1,"b":2}]]}"#,
            r#"{"":0,"":1}"#,
        ] {
            assert!(validate_json(text).is_err(), "{text}");
        }
    }

    #[test]
    fn compares_decoded_escaped_and_unicode_keys() {
        for text in [
            r#"{"a":1,"\u0061":2}"#,
            r#"{"/":1,"\/":2}"#,
            r#"{"é":1,"\u00e9":2}"#,
            r#"{"😀":1,"\ud83d\ude00":2}"#,
            r#"{"\"":1,"\u0022":2}"#,
        ] {
            assert!(validate_json(text).is_err(), "{text}");
        }
    }

    #[test]
    fn allows_identical_names_in_independent_objects() -> Result<()> {
        validate_json(r#"{"a":{"a":1},"b":[{"a":2},{"a":3}]}"#)?;
        validate_json(r#"[{"a":1},{"a":2}]"#)?;
        validate_json(r#"{"a":1,"A":2,"é":3,"e\u0301":4}"#)?;
        Ok(())
    }

    #[test]
    fn ignores_structural_characters_inside_strings() -> Result<()> {
        validate_json(r#"{"text":"{\"a\":1,\"a\":2}","a":"} [ : {"}"#)?;
        validate_json(r#"{"\\":1,"\"":2,"\u007b":3,"/":4}"#)?;
        validate_json("{\"a\" \r\n\t : 1, \"b\": [\"a\", \"a\"]}")?;
        Ok(())
    }

    #[test]
    fn retains_all_json_values_and_arbitrary_precision_numbers() -> Result<()> {
        for text in [
            "null",
            "true",
            "false",
            "1e9999",
            "184467440737095516160000",
            r#""a: { } [ ]""#,
            "[]",
            "{}",
            r#"{"n":-123.45e+9999,"array":[null,true,false,{},[]]}"#,
        ] {
            validate_json(text)?;
        }
        Ok(())
    }

    #[test]
    fn syntax_trailing_input_and_recursion_limits_remain_enforced() {
        for text in [
            "",
            "{",
            "{\"a\":}",
            "{\"a\":1,}",
            "{} {}",
            "{\"a\":NaN}",
            r#"{"\ud800":1}"#,
            r#"{"a":"unterminated}"#,
        ] {
            assert!(validate_json(text).is_err(), "{text}");
        }
        let nested = format!("{}0{}", "[".repeat(256), "]".repeat(256));
        assert!(validate_json(&nested).is_err());
    }

    #[test]
    fn duplicate_errors_report_location_without_echoing_data() {
        let result = validate_json(r#"{"private-name":1,"private-name":"private-value"}"#);
        match result {
            Ok(()) => assert!(false, "duplicate names must reject"),
            Err(error) => {
                let message = error.to_string();
                assert!(message.contains("duplicate JSON object key at UTF-8 byte"));
                assert!(!message.contains("private-name"));
                assert!(!message.contains("private-value"));
            }
        }
    }
}
