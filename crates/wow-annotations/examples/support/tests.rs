use super::*;
use std::{
    env,
    process::{Command, Stdio},
};
fn run(args: Vec<OsString>) -> Result<bool, Box<dyn std::error::Error>> {
    super::run(args, None)
}

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
fn correction_file(
    fixture: &Fixture,
    stale: bool,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    use wow_reference::native_corrections::*;
    let revision = String::from_utf8(git(&fixture.0, &["rev-parse", "HEAD"], 128)?)?
        .trim()
        .to_owned();
    let source = String::from_utf8(git(&fixture.0, &["show", "HEAD:API.lua"], LIMIT)?)?;
    let document = ingest_document(
        &revision,
        "API.lua",
        &source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )?;
    let normalized = wow_reference::native_model::normalize_document(&document);
    let field = normalized.systems[0]
        .as_ref()
        .map_err(|_| "fixture normalization")?
        .functions[0]
        .returns[0]
        .raw;
    let data = CorrectionSet {
        schema: SCHEMA.into(),
        version: 1,
        revision: if stale { "b".repeat(40) } else { revision },
        environment: "Mainline".into(),
        normalizer: NORMALIZER.into(),
        records: vec![Correction {
            id: "fixture-return".into(),
            target: Target {
                path: "API.lua".into(),
                registration: 0,
                projection: Projection::CallableField {
                    function: "Read".into(),
                    lane: Lane::Returns,
                    member: "value".into(),
                    property: Property::Nilable,
                },
            },
            expected_source_sha256: document.sha256().into(),
            expected_raw_sha256: raw_digest(field)?,
            before: Value::Absent,
            after: Value::Boolean(true),
            reviewer: "test".into(),
            rationale: "Synthetic driver test".into(),
            evidence: vec![Evidence {
                revision: "a".repeat(40),
                path: "fixture.lua".into(),
                sha256: source_digest(b"fixture"),
            }],
        }],
    };
    let path = fixture.0.join("corrections.json");
    fs::write(&path, serde_json::to_vec(&data)?)?;
    Ok(path)
}
#[test]
fn explicit_reviewed_correction_flows_through_git_driver() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let pack = correction_file(&fixture, false)?;
    let mut args = fixture.args("corrected");
    args.extend(["--corrections".into(), pack.into_os_string()]);
    assert!(!run(args)?);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("corrected/source-report.json"))?)?;
    assert_eq!(
        report["library"]["schema"],
        "wow-native-annotation-library/4"
    );
    assert_eq!(
        report["library"]["corrections"]["applications"][0]["status"],
        "applied"
    );
    assert!(
        fs::read_to_string(fixture.0.join("corrected/api-0000.lua"))?
            .contains("---@return boolean? value")
    );
    Ok(())
}
#[test]
fn expired_correction_makes_driver_partial_and_keeps_original_value()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let pack = correction_file(&fixture, true)?;
    let mut args = fixture.args("expired");
    args.extend(["--corrections".into(), pack.into_os_string()]);
    assert!(run(args)?);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("expired/source-report.json"))?)?;
    assert_eq!(
        report["library"]["corrections"]["applications"][0]["status"],
        "expired"
    );
    assert_eq!(report["status"], "partial");
    assert!(
        fs::read_to_string(fixture.0.join("expired/api-0000.lua"))?
            .contains("---@return boolean value")
    );
    Ok(())
}
#[test]
fn malformed_correction_input_never_creates_an_output_directory()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let path = fixture.0.join("corrections.json");
    fs::write(&path, b"{}")?;
    let mut args = fixture.args("invalid");
    args.extend(["--corrections".into(), path.into_os_string()]);
    assert!(run(args).is_err());
    assert!(!fixture.0.join("invalid").exists());
    Ok(())
}
#[test]
fn native_git_driver_emits_source_bound_local_receiver_class()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.0.join("API.lua"),
        r#"APIDocumentation:AddDocumentationTable({Name="SyntheticObjectAPI",Type="ScriptObject",Functions={{Name="Show"}}})"#,
    )?;
    fixture.command(&["add", "."])?;
    fixture.command(&["commit", "-m", "object fixture"])?;
    assert!(!run(fixture.args("out"))?);
    let text = fs::read_to_string(fixture.0.join("out/api-0000.lua"))?;
    assert!(
        text.starts_with("---@meta _\n---@class SyntheticObjectAPI\nlocal SyntheticObjectAPI = {}")
    );
    assert!(text.contains("function SyntheticObjectAPI:Show() end"));
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("out/source-report.json"))?)?;
    let maps = report["library"]["files"][0]["mappings"]
        .as_array()
        .ok_or("missing maps")?;
    assert_eq!(maps.len(), 2);
    assert_eq!(maps[0]["source"]["path"], "API.lua");
    assert_eq!(report["library"]["negative_authority"], false);
    Ok(())
}
fn alias_args(source: &Fixture, donor: &Fixture, output: &str) -> Vec<OsString> {
    let mut args = source.args(output);
    args.extend([
        "--alias-catalog".into(),
        donor.0.clone().into_os_string(),
        "HEAD".into(),
        "Aliases.lua".into(),
    ]);
    args
}
fn alias_fixture(text: &str) -> Result<Fixture, Box<dyn std::error::Error>> {
    let donor = Fixture::new()?;
    fs::write(donor.0.join("Aliases.lua"), text)?;
    donor.command(&["add", "."])?;
    donor.command(&["commit", "-m", "annotation catalog"])?;
    Ok(donor)
}
#[test]
fn alias_git_driver_binds_exact_committed_resource_not_dirty_worktree()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let donor = alias_fixture("---@meta _\n---@alias ExternalValue number|string\n")?;
    fs::write(
        donor.0.join("Aliases.lua"),
        "os.execute('never read or run')",
    )?;
    assert!(!run(alias_args(&fixture, &donor, "out"))?);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("out/source-report.json"))?)?;
    let library = &report["library"];
    assert_eq!(library["schema"], "wow-native-annotation-library/5");
    assert_ne!(
        library["revision"],
        library["aliases"]["source"]["revision"]
    );
    assert_eq!(
        library["aliases"]["authority"],
        "external_annotation_overlay"
    );
    assert_eq!(library["aliases"]["outcomes"][0]["status"], "emitted");
    assert!(
        fs::read_to_string(fixture.0.join("out/aliases-0001.lua"))?
            .contains("---@alias ExternalValue number|string")
    );
    assert!(!serde_json::to_string(library)?.contains(donor.0.to_str().ok_or("path")?));
    assert!(run(alias_args(&fixture, &donor, "out")).is_err());
    Ok(())
}
#[test]
fn alias_driver_and_correction_flags_compose_in_either_order()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let donor = alias_fixture("---@alias ExternalValue number\n")?;
    let pack = correction_file(&fixture, false)?;
    let mut reports = Vec::new();
    for (output, alias_first) in [("alias-first", true), ("correction-first", false)] {
        let mut args = fixture.args(output);
        let alias = alias_args(&fixture, &donor, output).split_off(5);
        let correction = vec!["--corrections".into(), pack.clone().into_os_string()];
        if alias_first {
            args.extend(alias);
            args.extend(correction);
        } else {
            args.extend(correction);
            args.extend(alias);
        }
        assert!(!run(args)?);
        let report: serde_json::Value = serde_json::from_slice(&fs::read(
            fixture.0.join(output).join("source-report.json"),
        )?)?;
        assert_eq!(
            report["library"]["schema"],
            "wow-native-annotation-library/5"
        );
        assert_eq!(
            report["library"]["corrections"]["applications"][0]["status"],
            "applied"
        );
        reports.push(report["library"].clone());
    }
    assert_eq!(reports[0], reports[1]);
    Ok(())
}
#[test]
fn missing_alias_dependency_keeps_valid_output_but_driver_is_partial()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let donor = alias_fixture("---@alias Valid number\n---@alias Broken UnknownTarget\n")?;
    assert!(run(alias_args(&fixture, &donor, "out"))?);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("out/source-report.json"))?)?;
    assert_eq!(report["status"], "partial");
    assert_eq!(report["input_failures"], serde_json::json!([]));
    let output = fs::read_to_string(fixture.0.join("out/aliases-0001.lua"))?;
    assert!(output.contains("---@alias Valid number"));
    assert!(!output.contains("---@alias Broken"));
    Ok(())
}
#[test]
fn invalid_catalog_and_cli_options_reject_before_output_creation()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let donor = alias_fixture("---@alias Valid number\nreturn {}\n")?;
    assert!(run(alias_args(&fixture, &donor, "out")).is_err());
    let args = alias_args(&fixture, &donor, "out");
    for mode in 0..5 {
        let mut bad = args.clone();
        match mode {
            0 => {
                bad.pop();
            }
            1 => {
                bad.extend(args[5..].iter().cloned());
            }
            2 => {
                bad[7] = "--help".into();
            }
            3 => {
                bad[8] = "../Aliases.lua".into();
            }
            _ => {
                bad[8] = "Missing.lua".into();
            }
        }
        assert!(run(bad).is_err());
        assert!(!fixture.0.join("out").exists());
    }
    Ok(())
}
#[cfg(unix)]
#[test]
fn alias_git_symlink_is_not_an_annotation_resource() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let donor = Fixture::new()?;
    fs::write(donor.0.join("Real.lua"), "---@alias Valid number\n")?;
    std::os::unix::fs::symlink("Real.lua", donor.0.join("Aliases.lua"))?;
    donor.command(&["add", "."])?;
    donor.command(&["commit", "-m", "symlink input"])?;
    assert!(run(alias_args(&fixture, &donor, "out")).is_err());
    assert!(!fixture.0.join("out").exists());
    Ok(())
}
