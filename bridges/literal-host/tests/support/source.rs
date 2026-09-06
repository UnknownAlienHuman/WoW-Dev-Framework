use serde_json::Value;
use std::{
    error::Error,
    ffi::OsString,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;
static NEXT: AtomicU64 = AtomicU64::new(0);
pub const SOURCE: &str = r#"APIDocumentation:AddDocumentationTable({Name="Synthetic",Type="System",Namespace="C_Synthetic",
Functions={{Name="Read",Returns={{Name="value",Type="bool"}}}},
Events={{Name="Changed",LiteralName="SYNTHETIC_CHANGED",Payload={{Name="value",Type="number"}}}},
Tables={{Name="Choice",Type="Enumeration",Fields={{Name="First",EnumValue=1}}},
{Name="Limits",Type="Constants",Values={{Name="Count",Value=3}}}}})"#;
pub struct Fixture(pub PathBuf);
impl Fixture {
    pub fn new() -> TestResult<Self> {
        let root = std::env::temp_dir().join(format!(
            "wdf-wasm-source-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let this = Self(root);
        for args in [
            &["init", "--initial-branch=main"][..],
            &["config", "user.name", "Synthetic test"],
            &["config", "user.email", "test@example.invalid"],
            &["config", "core.autocrlf", "false"],
        ] {
            this.git(args)?;
        }
        fs::write(this.0.join("API.lua"), SOURCE)?;
        fs::write(this.0.join("API.toc"), "API.lua\n")?;
        fs::write(
            this.0.join("Aliases.lua"),
            "---@meta _\n---@alias ExtraValue boolean|number\n",
        )?;
        this.git(&["add", "."])?;
        this.git(&["commit", "-qm", "synthetic source"])?;
        Ok(this)
    }
    pub fn git(&self, args: &[&str]) -> TestResult {
        if !Command::new("git")
            .arg("-C")
            .arg(&self.0)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success()
        {
            return Err("fixture git failed".into());
        }
        Ok(())
    }
    pub fn args(&self, output: &str) -> Vec<OsString> {
        let mut args = vec![
            self.0.clone().into_os_string(),
            "HEAD".into(),
            "API.toc".into(),
            "Mainline".into(),
            self.0.join(output).into_os_string(),
            "--alias-catalog".into(),
            self.0.clone().into_os_string(),
            "HEAD".into(),
            "Aliases.lua".into(),
        ];
        if self.0.join("corrections.json").exists() {
            args.extend([
                "--corrections".into(),
                self.0.join("corrections.json").into_os_string(),
            ]);
        }
        args
    }
    pub fn corrections(&self) -> TestResult {
        use wow_reference::{
            native::{ingest_document, source_digest},
            native_corrections::*,
            native_model::normalize_document,
        };
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.0)
            .args(["rev-parse", "HEAD"])
            .output()?;
        if !output.status.success() {
            return Err("fixture revision unavailable".into());
        }
        let revision = std::str::from_utf8(&output.stdout)?.trim().to_owned();
        let document = ingest_document(
            &revision,
            "API.lua",
            SOURCE,
            &source_digest(SOURCE.as_bytes()),
            &std::sync::atomic::AtomicBool::new(false),
        )?;
        let normalized = normalize_document(&document);
        let field = normalized.systems[0]
            .as_ref()
            .map_err(|_| "normalization failed")?
            .functions[0]
            .returns[0]
            .raw;
        let set = CorrectionSet {
            schema: SCHEMA.into(),
            version: 1,
            revision,
            environment: "Mainline".into(),
            normalizer: NORMALIZER.into(),
            records: vec![Correction {
                id: "synthetic-return".into(),
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
                before: wow_reference::native_corrections::Value::Absent,
                after: wow_reference::native_corrections::Value::Boolean(true),
                reviewer: "test".into(),
                rationale: "Synthetic integration correction".into(),
                evidence: vec![Evidence {
                    revision: "a".repeat(40),
                    path: "synthetic.lua".into(),
                    sha256: source_digest(b"fixture"),
                }],
            }],
        };
        fs::write(self.0.join("corrections.json"), serde_json::to_vec(&set)?)?;
        Ok(())
    }
    pub fn report(&self, output: &str) -> TestResult<Value> {
        Ok(serde_json::from_slice(&fs::read(
            self.0.join(output).join("source-report.json"),
        )?)?)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
