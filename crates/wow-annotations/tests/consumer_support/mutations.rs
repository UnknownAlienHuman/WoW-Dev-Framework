//! Real consumer negative controls: incomplete libraries cannot pass the gate.
use super::{Result, fixture, process::Executable, report};
use serde_json::{Value, json};
use std::{fs, path::Path};
pub fn run(executable: &Executable, root: &Path) -> Result<Vec<Value>> {
    let mut results = Vec::new();
    for mutation in ["erased-argument-type", "missing-library"] {
        let output = root.join(format!("{}-{mutation}", executable.kind.name()));
        fs::create_dir(&output)?;
        fs::create_dir(output.join("home"))?;
        let input = output.join("input");
        fixture::prepare(&input)?;
        let mut changed = 0;
        for entry in fs::read_dir(&input)? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid fixture path")?;
            if name.starts_with("api-") || name.starts_with("values-") {
                if mutation == "missing-library" {
                    fs::remove_file(entry.path())?;
                    changed += 1;
                } else {
                    let text = fs::read_to_string(entry.path())?;
                    let changed_text = text.replace("---@param id number", "---@param id any");
                    if text != changed_text {
                        fs::write(entry.path(), changed_text)?;
                        changed += 1;
                    }
                }
            }
        }
        if changed == 0 {
            return Err("mutation did not change an artifact".into());
        }
        let before = fixture::snapshot(&input)?;
        let (status, raw) = executable.check(&output)?;
        let diagnostics = report::normalize(executable.kind, &raw, &input)?;
        let detected = report::assert_behavior(&diagnostics, status).is_err();
        let unchanged = before == fixture::snapshot(&input)?;
        results.push(
            json!({"mutation":mutation,"detected":detected,"input_unchanged":unchanged,
            "input_sha256":before,"diagnostics":diagnostics}),
        );
    }
    Ok(results)
}
