//! Exact, independent Git inputs for the closed-string catalog consumer probe.
use super::{Result, source};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path, sync::atomic::AtomicBool};
use wow_reference::native::{ingest_document, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

const API: &str = r#"APIDocumentation:AddDocumentationTable({Name="CatalogProbe",Type="System",Namespace="C_CatalogProbe",Functions={
{Name="Choose",Arguments={{Name="choice",Type="ProbeChoice"}}},
{Name="Read",Returns={{Name="choice",Type="ProbeChoice"}}},
{Name="Optional",Arguments={{Name="choice",Type="OptionalProbeChoice"}}}
}})"#;
const CATALOG: &str = "---@meta _\n---@alias ProbeChoice\n---|\"FIRST\"\n---|'SECOND'\n---@alias OptionalProbeChoice ProbeChoice|nil\n";

pub(super) fn prepare(input: &Path) -> Result<Value> {
    let source_root = input.with_extension("source");
    let catalog_root = input.with_extension("catalog");
    let revision = checkout(&source_root, &[("API.toc", "API.lua\n"), ("API.lua", API)])?;
    let catalog_revision = checkout(&catalog_root, &[("Types.lua", CATALOG)])?;
    if revision == catalog_revision {
        return Err("catalog and documentation fixture identities unexpectedly coincide".into());
    }
    fs::write(source_root.join("API.toc"), "uncommitted.lua\n")?;
    fs::write(source_root.join("API.lua"), "not committed documentation\n")?;
    fs::write(
        catalog_root.join("Types.lua"),
        "not committed annotations\n",
    )?;
    if source::driver::run(
        vec![
            source_root.into_os_string(),
            revision.clone().into(),
            "API.toc".into(),
            "Mainline".into(),
            input.as_os_str().to_owned(),
            "--alias-catalog".into(),
            catalog_root.into_os_string(),
            catalog_revision.clone().into(),
            "Types.lua".into(),
        ],
        None,
    )? {
        return Err("catalog consumer source build is partial".into());
    }
    verify(input, &revision, &catalog_revision)?;
    Ok(
        json!({"revision":revision,"catalog_revision":catalog_revision,
        "catalog_sha256":source_digest(CATALOG.as_bytes())}),
    )
}

fn checkout(path: &Path, files: &[(&str, &str)]) -> Result<String> {
    fs::create_dir(path)?;
    fs::create_dir(path.join("home"))?;
    fs::create_dir(path.join("hooks"))?;
    fs::write(path.join("home/.gitconfig"), "")?;
    source::git(path, &["init", "--initial-branch=main", "--template="])?;
    for (name, text) in files {
        fs::write(path.join(name), text)?;
    }
    let mut args = vec!["add", "--"];
    args.extend(files.iter().map(|(name, _)| *name));
    source::git(path, &args)?;
    source::git(
        path,
        &["commit", "--no-gpg-sign", "-m", "catalog probe input"],
    )?;
    let revision = source::git(path, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    Ok(revision.trim().to_owned())
}

fn verify(input: &Path, revision: &str, catalog_revision: &str) -> Result<()> {
    let cancelled = AtomicBool::new(false);
    let documents = [ingest_document(
        revision,
        "API.lua",
        API,
        &source_digest(API.as_bytes()),
        &cancelled,
    )?];
    let catalog = ingest_alias_catalog(
        catalog_revision,
        "Types.lua",
        CATALOG,
        &source_digest(CATALOG.as_bytes()),
        &cancelled,
    )?;
    let expected = wow_annotations::native::project_with_alias_catalog(
        &documents,
        "Mainline",
        None,
        Some(&catalog),
        &cancelled,
    )?;
    let report: Value = serde_json::from_slice(&fs::read(input.join("source-report.json"))?)?;
    if expected.projection != "projected_with_sidecars"
        || report["schema"] != "wow-native-source-build/1"
        || report["revision"] != revision
        || report["selector"] != revision
        || report["environment"] != "Mainline"
        || report["freshness"] != "not_network_verified"
        || report["status"] != "projected_with_sidecars"
        || report["negative_authority"] != false
        || report["candidate_files"] != 1
        || report["admitted_files"] != 1
        || report["input_failures"] != json!([])
        || report["source_order"] != json!(["API.lua"])
        || report["toc"] != json!({"path":"API.toc","sha256":source_digest(b"API.lua\n")})
        || report["library"] != serde_json::to_value(&expected)?
    {
        return Err("catalog driver output does not match the exact committed fixture".into());
    }
    let mut names = BTreeSet::from(["source-report.json".to_owned()]);
    for file in &expected.files {
        names.insert(file.path.clone());
        if fs::read(input.join(&file.path))? != file.text.as_bytes() {
            return Err("catalog driver artifact bytes differ from its receipt".into());
        }
    }
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(input)? {
        let entry = entry?;
        let meta = fs::symlink_metadata(entry.path())?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err("unexpected catalog fixture entry".into());
        }
        actual.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid fixture filename")?,
        );
    }
    if actual != names {
        return Err("catalog fixture inventory mismatch".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consumer_support::Workspace;

    #[test]
    fn independent_committed_catalog_ignores_both_dirty_worktrees() -> Result<()> {
        let workspace = Workspace::new()?;
        let input = workspace.path.join("input");
        let identity = prepare(&input)?;
        assert_ne!(identity["revision"], identity["catalog_revision"]);
        assert_eq!(
            fs::read_to_string(input.with_extension("catalog").join("Types.lua"))?,
            "not committed annotations\n"
        );
        let report: Value = serde_json::from_slice(&fs::read(input.join("source-report.json"))?)?;
        assert_eq!(report["library"]["aliases"]["source"]["text"], CATALOG);
        assert_eq!(
            report["library"]["aliases"]["outcomes"][0]["status"],
            "emitted"
        );
        assert_eq!(
            report["library"]["aliases"]["outcomes"][1]["status"],
            "emitted"
        );
        Ok(())
    }

    #[test]
    fn wrong_resource_identity_and_changed_catalog_bytes_reject() -> Result<()> {
        let workspace = Workspace::new()?;
        let input = workspace.path.join("input");
        let identity = prepare(&input)?;
        let revision = identity["revision"].as_str().ok_or("revision")?;
        let catalog_revision = identity["catalog_revision"]
            .as_str()
            .ok_or("catalog revision")?;
        assert!(verify(&input, revision, revision).is_err());
        let report: Value = serde_json::from_slice(&fs::read(input.join("source-report.json"))?)?;
        let file = report["library"]["files"]
            .as_array()
            .ok_or("files")?
            .iter()
            .find(|f| {
                f["path"]
                    .as_str()
                    .is_some_and(|s| s.starts_with("aliases-"))
            })
            .ok_or("catalog artifact")?;
        fs::write(
            input.join(file["path"].as_str().ok_or("path")?),
            "-- changed\n",
        )?;
        assert!(verify(&input, revision, catalog_revision).is_err());
        Ok(())
    }
}
