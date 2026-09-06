use crate::{Result, git, manifest, skill};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(format: &str) -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "wdf-xtask-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let fixture = Self(root);
        fixture.command(&[
            "init",
            "--initial-branch=live",
            &format!("--object-format={format}"),
        ])?;
        fixture.command(&["config", "user.name", "Synthetic fixture"])?;
        fixture.command(&["config", "user.email", "test@example.invalid"])?;
        fixture.command(&["config", "core.autocrlf", "false"])?;
        Ok(fixture)
    }
    fn command(&self, args: &[&str]) -> Result<()> {
        if !Command::new("git")
            .arg("-C")
            .arg(&self.0)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success()
        {
            return Err("test Git setup failed".into());
        }
        Ok(())
    }
    fn put(&self, relative: &str, bytes: &[u8]) -> Result<()> {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
        fs::write(path, bytes)?;
        Ok(())
    }
    fn commit(&self) -> Result<()> {
        self.command(&["add", "."])?;
        self.command(&["commit", "-qm", "fixture"])
    }
    fn canonical(&self) -> Result<()> {
        self.put(
            skill::CANONICAL,
            b"---\nname: wow-dev\n---\n# Native workflow\n",
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn skill_check_is_read_only_and_write_is_idempotent() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    fixture.canonical()?;
    assert!(!skill::sync(&fixture.0, false)?);
    assert!(!fixture.0.join(skill::TARGETS[0]).exists());
    assert!(skill::sync(&fixture.0, true)?);
    assert!(skill::sync(&fixture.0, false)?);
    assert!(skill::sync(&fixture.0, true)?);
    for path in skill::TARGETS {
        assert_eq!(
            fs::read(fixture.0.join(path))?,
            fs::read(fixture.0.join(skill::CANONICAL))?
        );
    }
    assert!(!fixture.0.join(".wow-dev-skill.lock").exists());
    fixture.put(skill::TARGETS[1], b"stale")?;
    assert!(!skill::sync(&fixture.0, false)?);
    assert_eq!(fs::read(fixture.0.join(skill::TARGETS[1]))?, b"stale");
    Ok(())
}
#[test]
fn invalid_canonical_and_active_lock_fail_without_partial_updates() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    fixture.put(skill::CANONICAL, b"not a skill")?;
    assert!(skill::sync(&fixture.0, true).is_err());
    fixture.canonical()?;
    fixture.put(".wow-dev-skill.lock", b"another writer")?;
    assert!(skill::sync(&fixture.0, true).is_err());
    assert!(!fixture.0.join(skill::TARGETS[0]).exists());
    assert_eq!(
        fs::read(fixture.0.join(".wow-dev-skill.lock"))?,
        b"another writer"
    );
    Ok(())
}
#[cfg(unix)]
#[test]
fn skill_symlinks_are_rejected_before_any_write() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    fixture.canonical()?;
    std::os::unix::fs::symlink("/tmp", fixture.0.join(".agent"))?;
    assert!(skill::sync(&fixture.0, true).is_err());
    assert!(!fixture.0.join(skill::TARGETS[0]).exists());
    Ok(())
}
#[test]
fn source_paths_and_remote_refs_fail_closed() -> Result<()> {
    for path in ["", "../x", "a//b", "/x", "C:/x", "a\\b", "x\ny"] {
        assert!(manifest::validate_path(path).is_err());
    }
    manifest::validate_path("Interface/Test файл.lua")?;
    let hash = "a".repeat(40);
    assert_eq!(
        git::parse_remote(&format!("{hash}\trefs/heads/live"), "refs/heads/live")?,
        hash
    );
    for value in [
        format!("{hash}\trefs/heads/ptr"),
        format!("{hash}\trefs/heads/live\n{hash}\trefs/heads/live"),
        "bad".into(),
    ] {
        assert!(git::parse_remote(&value, "refs/heads/live").is_err());
    }
    Ok(())
}
#[test]
fn raw_snapshot_inventory_tamper_stale_no_clobber_and_formats() -> Result<()> {
    for format in ["sha1", "sha256"] {
        let fixture = Fixture::new(format)?;
        fixture.put("version.txt", b"99.0.0.12345\n")?;
        fixture.put(
            "Interface/AddOns/Blizzard_APIDocumentationGenerated/Test.lua",
            b"-- $Format:%H$\nlocal fixture = 1\n",
        )?;
        fixture.put("Interface/Test.xml", b"<Ui/>\n")?;
        fixture.put("Interface/Test.toc", b"Test.xml\n")?;
        fixture.put("Interface/Test.xsd", b"<schema/>\n")?;
        fixture.put("asset.png", b"excluded")?;
        fixture.put(
            ".gitattributes",
            b"*.lua export-subst\nversion.txt export-ignore\n",
        )?;
        fixture.commit()?;
        let value = manifest::build(&fixture.0, "HEAD", "live")?;
        assert_eq!(value["source"]["git_object_format"], format);
        assert_eq!(value["coverage"]["included_files"], 5);
        assert_eq!(value["coverage"]["excluded_files"], 2);
        assert_eq!(value["coverage"]["kind_generated_api"], 1);
        assert_eq!(value["source"]["version"], "99.0.0.12345");
        let output = fixture.0.join("manifest.json");
        manifest::write_new(&output, &value)?;
        assert!(manifest::write_new(&output, &value).is_err());
        fixture.put("version.txt", b"dirty worktree")?;
        assert_eq!(manifest::verify(&output, &fixture.0, Some("HEAD"))?, 0);
        let mut tampered = value.clone();
        tampered["coverage"]["included_files"] = serde_json::json!(0);
        fs::write(&output, serde_json::to_vec(&tampered)?)?;
        assert!(manifest::verify(&output, &fixture.0, None).is_err());
        fs::write(&output, serde_json::to_vec(&value)?)?;
        fixture.command(&["add", "version.txt"])?;
        fixture.command(&["commit", "-qm", "advance"])?;
        assert_eq!(manifest::verify(&output, &fixture.0, Some("live"))?, 3);
        assert_eq!(manifest::verify(&output, &fixture.0, None)?, 0);
    }
    Ok(())
}
#[test]
fn manifest_requires_version_and_bounds_git_output() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    fixture.put("API.lua", b"fixture")?;
    fixture.commit()?;
    assert!(manifest::build(&fixture.0, "HEAD", "live").is_err());
    assert!(git::run(&fixture.0, &["show", "HEAD:API.lua"], None, 2).is_err());
    assert!(git::resolve(&fixture.0, "--help").is_err());
    Ok(())
}
#[cfg(unix)]
#[test]
fn inventory_rejects_symlinks_even_outside_selected_extensions() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    fixture.put("version.txt", b"99.0.0.1")?;
    std::os::unix::fs::symlink("/tmp", fixture.0.join("asset"))?;
    fixture.commit()?;
    assert!(manifest::build(&fixture.0, "HEAD", "live").is_err());
    Ok(())
}
#[test]
fn repository_guard_checks_tracked_and_untracked_files() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    fixture.put("README.md", b"native project")?;
    fixture.commit()?;
    crate::repository::check(&fixture.0)?;
    fixture.put("new.PY", b"unwanted")?;
    assert!(crate::repository::check(&fixture.0).is_err());
    fixture.command(&["add", "new.PY"])?;
    assert!(crate::repository::check(&fixture.0).is_err());
    Ok(())
}

fn artifact(fixture: &Fixture) -> Result<serde_json::Value> {
    let text = "---@meta _\nfunction Synthetic() end\n";
    let hash = format!("sha256:{}", manifest::digest(text.as_bytes()));
    let report = serde_json::json!({"schema":"wow-native-source-build/1","revision":"a".repeat(40),"status":"projected_with_sidecars","negative_authority":false,"source_order":["API.lua"],"candidate_files":1,"admitted_files":1,"input_failures":[],"library":{"schema":"wow-native-annotation-library/3","revision":"a".repeat(40),"projection":"projected_with_sidecars","negative_authority":false,"sources":[{"path":"API.lua","revision":"a".repeat(40),"source_bytes":100,"sha256":format!("sha256:{}", "b".repeat(64))}],"issues":[],"files":[{"path":"api-0000.lua","text":text,"sha256":hash,"mappings":[{"generated":{"start":0,"end":text.len()},"source":{"path":"API.lua","sha256":format!("sha256:{}", "b".repeat(64)),"span":{"start":0,"end":100}}}]}]}});
    fixture.put("output/api-0000.lua", text.as_bytes())?;
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    Ok(report)
}
#[test]
fn native_artifact_verifier_detects_final_byte_tampering() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    artifact(&fixture)?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 0);
    fixture.put("output/api-0000.lua", b"changed")?;
    assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    Ok(())
}
#[test]
fn native_artifact_verifier_detects_false_clean_and_unlisted_files() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    let original = artifact(&fixture)?;
    let mut report = original.clone();
    report["library"]["issues"] = serde_json::json!([{"code":"unsupported"}]);
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert!(crate::library::verify(&fixture.0.join("output"), false).is_err());
    report["status"] = serde_json::json!("partial");
    report["library"]["projection"] = serde_json::json!("partial");
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 3);
    fixture.put("output/unlisted.lua", b"unlisted")?;
    assert!(crate::library::verify(&fixture.0.join("output"), false).is_err());
    Ok(())
}
#[test]
fn native_artifact_verifier_rejects_bad_mapping_and_source_count() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    let original = artifact(&fixture)?;
    let mut report = original.clone();
    report["library"]["files"][0]["mappings"][0]["source"]["span"]["end"] = serde_json::json!(101);
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert!(crate::library::verify(&fixture.0.join("output"), false).is_err());
    let mut report = original;
    report["candidate_files"] = serde_json::json!(2);
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert!(crate::library::verify(&fixture.0.join("output"), false).is_err());
    Ok(())
}

fn corrected_artifact(fixture: &Fixture, status: &str) -> Result<serde_json::Value> {
    let mut report = artifact(fixture)?;
    let source_digest = format!("sha256:{}", "b".repeat(64));
    let raw_digest = format!("sha256:{}", "c".repeat(64));
    let target =
        serde_json::json!({"path":"API.lua","registration":0,"projection":{"kind":"widget_owner"}});
    let before = serde_json::json!({"kind":"text","value":"Original"});
    let after = serde_json::json!({"kind":"text","value":"Renamed"});
    let set = serde_json::json!({"schema":"wow-native-corrections/1","version":1,"revision":"a".repeat(40),"environment":"Mainline","normalizer":"native-model/1","records":[{"id":"one","target":target,"expected_source_sha256":source_digest,"expected_raw_sha256":raw_digest,"before":before,"after":after,"reviewer":"synthetic","rationale":"synthetic report test","evidence":[]} ]});
    let id = format!("sha256:{}", manifest::digest(&serde_json::to_vec(&set)?));
    report["library"]["schema"] = serde_json::json!("wow-native-annotation-library/4");
    report["library"]["corrections"] = serde_json::json!({"schema":"wow-native-correction-applications/1","corrections":{"id":id,"set":set},"applications":[{"correction_id":"one","target":target,"status":status,"before":before,"after":if status=="applied" {after} else {serde_json::Value::Null},"observed_source_sha256":source_digest,"observed_raw_sha256":raw_digest}]});
    if status == "expired" {
        report["status"] = serde_json::json!("partial");
        report["library"]["projection"] = serde_json::json!("partial");
    }
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    Ok(report)
}
#[test]
fn native_v4_verifier_checks_correction_identity_and_preserves_v3() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    let mut report = corrected_artifact(&fixture, "applied")?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 0);
    report["library"]["corrections"]["corrections"]["set"]["version"] = serde_json::json!(2);
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    artifact(&fixture)?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 0);
    Ok(())
}
#[test]
fn expired_corrections_cannot_be_hidden_behind_empty_projection_issues() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    let mut report = corrected_artifact(&fixture, "expired")?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 3);
    report["status"] = serde_json::json!("projected_with_sidecars");
    report["library"]["projection"] = serde_json::json!("projected_with_sidecars");
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    Ok(())
}
#[test]
fn correction_report_cannot_drop_outcomes_or_claim_changed_replacement() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    for mode in 0..3 {
        let mut report = corrected_artifact(&fixture, "applied")?;
        if mode == 0 {
            report["library"]["corrections"]["applications"] = serde_json::json!([]);
        } else if mode == 1 {
            report["library"]["corrections"]["applications"][0]["after"] = serde_json::Value::Null;
        } else {
            report["library"]["corrections"]["applications"][0]["status"] =
                serde_json::json!("unknown");
        }
        fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
        assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    }
    Ok(())
}

fn aliased_artifact(
    fixture: &Fixture,
    base: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    use serde_json::json;
    let mut report = match base {
        Some(base) => base,
        None => artifact(fixture)?,
    };
    let raw = "---@meta _\n---@alias ExternalValue number|string\n";
    let start = raw.find("---@alias").ok_or("alias fixture")?;
    let span = json!({"start":start,"end":raw.len()-1});
    let hash = format!("sha256:{}", manifest::digest(raw.as_bytes()));
    let link =
        json!({"scope":"annotation_alias_catalog","path":"API.lua","sha256":hash,"span":span});
    report["library"]["schema"] = json!("wow-native-annotation-library/5");
    report["library"]["aliases"] = json!({"schema":"wow-native-alias-projection/1","authority":"external_annotation_overlay","source":{"schema":"wow-native-alias-resource/1","revision":"d".repeat(40),"path":"API.lua","sha256":hash,"source_bytes":raw.len(),"text":raw,"aliases":[{"name":"ExternalValue","terms":["number","string"],"syntax_error":false,"span":span}]},"outcomes":[{"ordinal":0,"name":"ExternalValue","status":"emitted"}],"limitations":["synthetic artifact test, not semantic certification"]});
    report["library"]["files"].as_array_mut().ok_or("files")?.push(json!({"path":"aliases-0001.lua","text":raw,"sha256":hash,"mappings":[{"granularity":"declaration","generated":span,"source":link}]}));
    fixture.put("output/aliases-0001.lua", raw.as_bytes())?;
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    Ok(report)
}
#[test]
fn alias_artifact_has_separate_identity_even_when_paths_overlap() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    let original = aliased_artifact(&fixture, None)?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 0);
    for mode in 0..4 {
        let mut report = original.clone();
        let blizzard_hash = report["library"]["sources"][0]["sha256"].clone();
        let mapping = &mut report["library"]["files"][1]["mappings"][0]["source"];
        match mode {
            0 => {
                mapping.as_object_mut().ok_or("mapping")?.remove("scope");
            }
            1 => {
                mapping["scope"] = serde_json::json!("other");
            }
            2 => {
                mapping["path"] = serde_json::json!("Other.lua");
            }
            _ => {
                mapping["sha256"] = blizzard_hash;
            }
        }
        fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
        assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    }
    Ok(())
}
#[test]
fn alias_verifier_rejects_raw_identity_outcome_and_mapping_tampering() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    let original = aliased_artifact(&fixture, None)?;
    for mode in 0..9 {
        let mut report = original.clone();
        let aliases = &mut report["library"]["aliases"];
        match mode {
            0 => aliases["source"]["text"] = serde_json::json!("changed"),
            1 => aliases["source"]["revision"] = serde_json::json!("live"),
            2 => aliases["authority"] = serde_json::json!("source_confirmed"),
            3 => aliases["outcomes"] = serde_json::json!([]),
            4 => aliases["outcomes"][0]["ordinal"] = serde_json::json!(1),
            5 => aliases["outcomes"][0]["name"] = serde_json::json!("Impostor"),
            6 => aliases["source"]["aliases"][0]["terms"] = serde_json::json!(["any"]),
            7 => report["library"]["files"][1]["mappings"] = serde_json::json!([]),
            _ => {
                let mapping = report["library"]["files"][1]["mappings"][0].clone();
                report["library"]["files"][1]["mappings"]
                    .as_array_mut()
                    .ok_or("mappings")?
                    .push(mapping);
            }
        }
        fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
        assert!(
            crate::library::verify(&fixture.0.join("output"), true).is_err(),
            "mutation {mode}"
        );
    }
    Ok(())
}
#[test]
fn rejected_alias_outcome_requires_issue_and_cannot_be_false_clean() -> Result<()> {
    use serde_json::json;
    let fixture = Fixture::new("sha1")?;
    let mut report = aliased_artifact(&fixture, None)?;
    let source = report["library"]["files"][1]["mappings"][0]["source"].clone();
    report["library"]["files"]
        .as_array_mut()
        .ok_or("files")?
        .pop();
    fs::remove_file(fixture.0.join("output/aliases-0001.lua"))?;
    report["library"]["aliases"]["outcomes"][0]["status"] = json!("source_name_conflict");
    report["library"]["issues"] = json!([{"code":"source_name_conflict","source":source}]);
    report["status"] = json!("partial");
    report["library"]["projection"] = json!("partial");
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 3);
    for mode in 0..3 {
        let mut invalid = report.clone();
        match mode {
            0 => invalid["library"]["issues"] = json!([]),
            1 => {
                invalid["status"] = json!("projected_with_sidecars");
                invalid["library"]["projection"] = json!("projected_with_sidecars");
            }
            _ => invalid["library"]["aliases"]["outcomes"][0]["status"] = json!("emitted"),
        }
        fixture.put("output/source-report.json", &serde_json::to_vec(&invalid)?)?;
        assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    }
    Ok(())
}
#[test]
fn alias_v5_preserves_correction_blockers_and_cannot_downgrade_schema() -> Result<()> {
    let fixture = Fixture::new("sha1")?;
    let base = corrected_artifact(&fixture, "expired")?;
    let original = aliased_artifact(&fixture, Some(base))?;
    assert_eq!(crate::library::verify(&fixture.0.join("output"), true)?, 3);
    for schema in [
        "wow-native-annotation-library/3",
        "wow-native-annotation-library/4",
    ] {
        let mut report = original.clone();
        report["library"]["schema"] = serde_json::json!(schema);
        fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
        assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    }
    let mut report = original;
    report["library"]
        .as_object_mut()
        .ok_or("library")?
        .remove("aliases");
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert!(crate::library::verify(&fixture.0.join("output"), true).is_err());
    Ok(())
}

#[test]
fn selected_library_composes_aliases_corrections_and_required_module() -> Result<()> {
    use serde_json::json;
    let fixture = Fixture::new("sha1")?;
    let base = corrected_artifact(&fixture, "expired")?;
    let mut report = aliased_artifact(&fixture, Some(base))?;
    let module = format!("sha256:{}", "c".repeat(64));
    report["library"]["schema"] = json!("wow-native-annotation-library/6");
    report["library"]["literal_execution"] = json!({"schema":"wow-literal-execution/1","module":{"sha256":module,"epoch":0},"calls":[],"artifacts":[]});
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert_eq!(
        crate::library::verify_with_module(&fixture.0.join("output"), true, Some(&module))?,
        3
    );
    assert!(
        crate::library::verify_with_module(
            &fixture.0.join("output"),
            true,
            Some(&format!("sha256:{}", "d".repeat(64)))
        )
        .is_err()
    );
    report["status"] = json!("projected_with_sidecars");
    report["library"]["projection"] = json!("projected_with_sidecars");
    fixture.put("output/source-report.json", &serde_json::to_vec(&report)?)?;
    assert!(
        crate::library::verify_with_module(&fixture.0.join("output"), true, Some(&module)).is_err()
    );
    Ok(())
}
