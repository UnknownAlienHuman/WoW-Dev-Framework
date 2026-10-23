//! Actual consumer feedback for independently selected Ketho-style string types.
use super::{Result, catalog_source, fixture, process::Executable, report};
use serde_json::{Value, json};
use std::{fs, path::Path};

const POSITIVE: &str = r#"C_CatalogProbe.Choose("FIRST")
C_CatalogProbe.Choose("SECOND")
C_CatalogProbe.Optional(nil)
C_CatalogProbe.Optional("FIRST")
local choice = C_CatalogProbe.Read()
C_CatalogProbe.Choose(choice)
---@type string
local text = choice
print(text:upper())
"#;
const NEGATIVES: &[(&str, &str, u64, &str)] = &[
    (
        "invalid-literal.lua",
        "C_CatalogProbe.Choose(\"THIRD\")\n",
        0,
        "param-type-mismatch",
    ),
    (
        "invalid-number.lua",
        "C_CatalogProbe.Choose(42)\n",
        0,
        "param-type-mismatch",
    ),
    (
        "invalid-return.lua",
        "---@type number\nlocal wrong = C_CatalogProbe.Read()\nprint(wrong)\n",
        1,
        "assign-type-mismatch",
    ),
    (
        "invalid-optional.lua",
        "C_CatalogProbe.Optional(false)\n",
        0,
        "param-type-mismatch",
    ),
];

pub fn run(executable: &Executable, root: &Path) -> Result<Value> {
    let mut checks = Vec::new();
    for widened in [false, true] {
        let mode = if widened { "widened" } else { "closed" };
        let output = root.join(format!("{}-catalog-{mode}", executable.kind.name()));
        fs::create_dir(&output)?;
        fs::create_dir(output.join("home"))?;
        let input = output.join("input");
        let identity = catalog_source::prepare(&input)?;
        if widened {
            widen(&input)?;
        }
        fs::write(input.join("positive.lua"), POSITIVE)?;
        for (name, text, _, _) in NEGATIVES {
            fs::write(input.join(name), text)?;
        }
        // Reuse the existing explicit JSON configuration bytes, not user settings.
        for name in ["emmy.json", "luals.json"] {
            fs::copy(root.join("input").join(name), input.join(name))?;
        }
        let before = fixture::snapshot(&input)?;
        let (status, raw) = executable.check(&output)?;
        let diagnostics = report::normalize(executable.kind, &raw, &input)?;
        let assertion = assert_behavior(&diagnostics, status, widened);
        let unchanged = before == fixture::snapshot(&input)?;
        checks.push(json!({"mode":mode,"source":identity,"exit":status,
            "input_sha256":before,"input_unchanged":unchanged,"diagnostics":diagnostics,
            "passed":assertion.is_ok()&&unchanged,"failure":assertion.err().map(|e|e.to_string())}));
    }
    Ok(
        json!({"scope":"synthetic-native-string-enum/1","negative_cases":NEGATIVES.len(),
        "mutation":"widen one closed alias to string; lose exactly the invalid-literal check",
        "passed":checks.iter().all(|c|c["passed"]==true),"checks":checks}),
    )
}

fn widen(input: &Path) -> Result<()> {
    const BEFORE: &str = "---@alias ProbeChoice \"FIRST\"|\"SECOND\"\n";
    const AFTER: &str = "---@alias ProbeChoice string\n";
    let mut candidates = Vec::new();
    let mut matches = 0;
    for entry in fs::read_dir(input)? {
        let entry = entry?;
        if !entry.file_name().to_string_lossy().starts_with("aliases-") {
            continue;
        }
        let text = fs::read_to_string(entry.path())?;
        let count = text.matches(BEFORE).count();
        matches += count;
        if count != 0 {
            candidates.push((entry.path(), text.replace(BEFORE, AFTER)));
        }
    }
    if matches != 1 {
        return Err("catalog mutation requires exactly one closed alias".into());
    }
    for (path, text) in candidates {
        fs::write(path, text)?;
    }
    Ok(())
}

fn assert_behavior(records: &[Value], status: i32, widened: bool) -> Result<()> {
    if status != 1 {
        return Err("catalog negative run must return diagnostics with exit 1".into());
    }
    if records.iter().any(|d| {
        d["file"]
            .as_str()
            .is_none_or(|file| !NEGATIVES.iter().any(|(name, _, _, _)| *name == file))
    }) {
        return Err("catalog library or positive fixture has diagnostics".into());
    }
    for (name, _, line, code) in NEGATIVES {
        if widened && *name == "invalid-literal.lua" {
            if records.iter().any(|d| d["file"] == *name) {
                return Err("widened alias still reports the closed-literal rejection".into());
            }
        } else if !records.iter().any(|d| {
            d["file"] == *name
                && d["code"] == *code
                && d["range"]["start"]["line"] == *line
                && d["severity"].as_u64().is_some_and(|s| (1..=2).contains(&s))
        }) {
            return Err(format!("missing required {code} at {name}:{}", line + 1).into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consumer_support::Workspace;

    fn expected() -> Vec<Value> {
        NEGATIVES
            .iter()
            .map(|(name, _, line, code)| {
                json!({"file":name,"code":code,"severity":2,
                    "range":{"start":{"line":line,"character":0}}})
            })
            .collect()
    }

    #[test]
    fn every_catalog_negative_and_clean_generated_file_is_required() -> Result<()> {
        let expected = expected();
        assert_behavior(&expected, 1, false)?;
        for index in 0..expected.len() {
            let mut missing = expected.clone();
            missing.remove(index);
            assert!(assert_behavior(&missing, 1, false).is_err());
            let mut unrelated = expected.clone();
            unrelated[index]["code"] = json!("unrelated-warning");
            assert!(assert_behavior(&unrelated, 1, false).is_err());
        }
        for name in ["positive.lua", "api-0000.lua", "aliases-0001.lua"] {
            let mut extra = expected.clone();
            extra.push(json!({"file":name,"severity":4,"code":"undefined-doc-name"}));
            assert!(assert_behavior(&extra, 1, false).is_err());
        }
        Ok(())
    }

    #[test]
    fn widening_must_lose_exactly_one_negative_not_all_type_feedback() -> Result<()> {
        let mut widened = expected();
        assert!(assert_behavior(&widened, 1, true).is_err());
        widened.remove(0);
        assert_behavior(&widened, 1, true)?;
        for index in 0..widened.len() {
            let mut missing = widened.clone();
            missing.remove(index);
            assert!(assert_behavior(&missing, 1, true).is_err());
        }
        for status in [0, 1, 2, -1] {
            assert!(assert_behavior(&[], status, true).is_err());
            if status != 1 {
                assert!(assert_behavior(&widened, status, true).is_err());
            }
        }
        Ok(())
    }

    #[test]
    fn mutation_changes_one_alias_and_cannot_repeat_silently() -> Result<()> {
        let workspace = Workspace::new()?;
        let input = workspace.path.join("input");
        catalog_source::prepare(&input)?;
        let before = fixture::snapshot(&input)?;
        widen(&input)?;
        let after = fixture::snapshot(&input)?;
        assert_eq!(before.len(), after.len());
        assert_eq!(
            before
                .iter()
                .filter(|(k, v)| after.get(*k) != Some(*v))
                .count(),
            1
        );
        assert!(widen(&input).is_err());
        Ok(())
    }
}
